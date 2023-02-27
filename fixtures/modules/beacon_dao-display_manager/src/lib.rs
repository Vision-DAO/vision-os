use beacon_dao_dom::{create_element, eval_js};
use beacon_dao_fetch::{fetch, OptionsBuilder, Response};
use beacon_dao_ipfs::{
	change_rpc_endpoint as change_endpoint_ipfs, get_rpc_endpoint as get_endpoint_ipfs,
};
use beacon_dao_logger_manager::info;
use beacon_dao_web3::{
	change_endpoint, eth_call, get_endpoint, BlockSelector, Error, TransactionCall,
	DEFAULT_NETWORKS,
};
use ethabi::{Contract, Token};
use serde::{Deserialize, Serialize};
use std::sync::{
	atomic::{AtomicUsize, Ordering},
	Arc, Mutex, RwLock,
};
use vision_derive::{
	beacon_dao_allocator::{allocate, grow, len, read, write},
	with_bindings,
};
use vision_utils::{
	actor::{address, spawn_actor_from},
	types::{
		Address, Callback, ALLOCATOR_ADDR, DOM_ADDR, EXIT_FAILURE, EXIT_SUCCESS, FETCH_ADDR,
		IPFS_ADDR, LOGGER_ADDR, PERM_ADDR, PERM_AGENT_ADDR, WEB3_ADDR,
	},
};

/// Kinds of dialogues supported by the display manager.
#[derive(Serialize, Deserialize)]
pub enum DialogueKind {
	/// A dialogue with no options
	Alert,

	/// A dialogue with only one option
	Affirm(String),

	/// A dialogue with two options
	Choice(String, String),
}

/// Spawns an actor from the given bytes
fn spawn_bytes(bytes: Arc<Vec<u8>>, callback: Callback<u8>) {
	allocate(
		ALLOCATOR_ADDR,
		Callback::new(|cell_addr| {
			grow(
				cell_addr,
				bytes.as_ref().len() as u32,
				Callback::new(move |_| {
					let remaining = Arc::new(AtomicUsize::new(bytes.len()));

					for (i, b) in bytes.as_ref().iter().enumerate() {
						let rem = remaining.clone();
						let cb = callback.clone();
						write(
							cell_addr,
							i as u32,
							*b,
							Callback::new(move |_| {
								// Last byte written. Spawn an actor from the bytes
								if rem.fetch_sub(1, Ordering::SeqCst) == 1 {
									spawn_actor_from(cell_addr);

									cb.call(EXIT_SUCCESS as u8);
								}
							}),
						);
					}
				}),
			);
		}),
	);
}

/// Loads the config profile at the specified Ethereum address.
#[no_mangle]
#[with_bindings]
pub extern "C" fn handle_login_as(from: Address, username: String, callback: Callback<u32>) {
	// Login as the default 'guest' user
	if username.is_empty() {
		// TODO: Put some address here

		return;
	}

	// TODO: Resolve ENS name
	// TODO: Resolve modules at that address
	// Get the IPFS address of the metadata associated with the user account
	// by calling the HasMetadata interface
	let contract = if let Ok(contract) = serde_json::from_str::<Contract>(include_str!(
		"../../../../abi/contracts/interfaces/IHasMetadata.sol/IHasMetadata.json"
	)) {
		contract
	} else {
		callback.call(EXIT_FAILURE);

		return;
	};

	let calldata = if let Ok(v) = contract.function("ipfsAddr").and_then(|f| {
		f.encode_input(&[])
			.map(|bytes| format!("0x{}", hex::encode(bytes)))
	}) {
		v
	} else {
		callback.call(EXIT_FAILURE);

		return;
	};

	eth_call(
		WEB3_ADDR,
		TransactionCall {
			from: None,
			to: username,
			gas: None,
			gasPrice: None,
			value: None,
			data: Some(calldata),
		},
		BlockSelector::Latest,
		Callback::new(move |res: Result<String, Error>| {
			// Convert 0x.. to raw bytes
			let bytes = if let Some(v) = res.ok().and_then(|v| hex::decode(&v[2..]).ok()) {
				v
			} else {
				callback.call(EXIT_FAILURE);
				return;
			};

			// Use ABI to get CID
			let cid = if let Some(cid) = contract
				.function("ipfsAddr")
				.ok()
				.and_then(|f| f.decode_output(bytes.as_slice()).ok())
				.and_then(|tokens| match tokens.get(0) {
					Some(Token::String(s)) => Some(s.clone()),
					_ => None,
				}) {
				cid
			} else {
				callback.call(EXIT_FAILURE);
				return;
			};

			info(LOGGER_ADDR, String::from(cid), Callback::new(|_| {}));
		}),
	)
}

/// System dialogue callbacks.
pub static TASKS: RwLock<Vec<Option<Callback<u32>>>> = RwLock::new(Vec::new());

/// Displays a dialogue on the screen with the given title, description, and nature.
/// Calls the callback when the dialogue is dismissed, with a return value
/// of 0 representing standard dismissal, and 1 representing the user choosing the
/// right binary choice, and 2 representing an error.
#[no_mangle]
#[with_bindings]
pub extern "C" fn handle_system_dialogue(
	from: Address,
	title: String,
	description: String,
	kind: DialogueKind,
	callback: Callback<u32>,
) {
	if from != PERM_ADDR && from != address() && from != PERM_AGENT_ADDR {
		callback.call(2);

		return;
	}

	// Register the user's callback for later
	let slot = {
		let mut slots = if let Ok(lock) = TASKS.write() {
			lock
		} else {
			callback.call(2);

			return;
		};

		let slot = slots.len();
		slots.push(Some(callback));

		slot
	};

	match kind {
		DialogueKind::Choice(yes, no) => create_element(
			DOM_ADDR,
			String::from("div"),
			include_str!("./dialogue/dialogue.html")
				.to_owned()
				.replace("#yes#", &yes)
				.replace("#no#", &no)
				.replace("#title#", &title)
				.replace("#desc#", &description),
			Callback::new(move |_| {
				eval_js(
					DOM_ADDR,
					include_str!("./dialogue/dialogue.js")
						.to_owned()
						.replace("#cbid#", &slot.to_string()),
					Callback::new(|_| {}),
				);
			}),
		),
		// TODO:
		_ => unimplemented!(),
	}
}

/// Handle the user clicking on yes or no, and return that status code to the end user.
#[no_mangle]
#[with_bindings]
pub extern "C" fn handle_system_dialogue_resp(
	from: Address,
	value: u32,
	cb_id: u32,
	cb: Callback<u32>,
) {
	let task = if let Some(task) = TASKS
		.write()
		.ok()
		.and_then(|mut tasks| tasks.get_mut(cb_id as usize).and_then(|task| task.take()))
	{
		task
	} else {
		cb.call(1);

		return;
	};

	task.call(value);
}

#[no_mangle]
pub extern "C" fn handle_display_login(from: Address) {
	create_element(
		DOM_ADDR,
		String::from("div"),
		include_str!("./index.html").to_owned(),
		Callback::new(|_| {
			eval_js(
				DOM_ADDR,
				include_str!("./index.js").to_owned(),
				Callback::new(|_| {}),
			);
		}),
	);

	// Add a task bar with some basic information on it
	create_element(
		DOM_ADDR,
		String::from("div"),
		include_str!("./taskbar/taskbar.html").to_owned(),
		Callback::new(|_| {
			eval_js(
				DOM_ADDR,
				include_str!("./taskbar/taskbar.js").to_owned(),
				Callback::new(|_| {}),
			);
		}),
	);
}

/// Displays the network chooser dialogue.
#[no_mangle]
pub extern "C" fn handle_change_network(from: Address, nonce: usize, callback: Callback<u32>) {
	// Make the current network the bolded option
	get_endpoint(
		WEB3_ADDR,
		Callback::new(move |curr_network| {
			let curr_net_index = DEFAULT_NETWORKS
				.iter()
				.position(|x| x == &curr_network)
				.unwrap();

			create_element(
				DOM_ADDR,
				String::from("div"),
				include_str!("./netdialogue/netdialogue.html")
					.to_owned()
					.replace(
						"#choices#",
						&DEFAULT_NETWORKS.iter().enumerate().fold(String::new(), |acc, (i, net)| {
							format!(
								"{}\n<p class=\"netChoice\" style=\"margin: 0; margin-bottom: 0.5em; cursor: pointer; background-color: #8241BA; text-transform: uppercase; padding: 0.5em; border-radius: 0.25em; transition: 0.3s{}\">{}</p>",
								acc, if i == curr_net_index { "; font-weight: bold" } else { "" }, net.name
							)
						}),
					)
					.replace("#curr#", &curr_net_index.to_string()),
				Callback::new(move |_| {
					eval_js(
						DOM_ADDR,
						include_str!("./netdialogue/netdialogue.js").to_owned().replace("#curr#", &curr_net_index.to_string()),
						Callback::new(|_| {}),
					);
				}),
			);
		}),
	);
}

/// Displays the endpoint chooser dialogue.
#[no_mangle]
pub extern "C" fn handle_change_ipfs_endpoint(
	from: Address,
	nonce: usize,
	callback: Callback<u32>,
) {
	get_endpoint_ipfs(
		IPFS_ADDR,
		Callback::new(move |curr: String| {
			create_element(
				DOM_ADDR,
				String::from("div"),
				include_str!("./netdialogue/ipfsdialogue.html")
					.to_owned()
					.replace("#curr#", &curr),
				Callback::new(move |_| {
					eval_js(
						DOM_ADDR,
						include_str!("./netdialogue/ipfsdialogue.js").to_owned(),
						Callback::new(|_| {}),
					);
				}),
			);
		}),
	);
}

/// Changes the web3 adapter network according to the chosen network.
#[no_mangle]
pub extern "C" fn handle_do_change_network(from: Address, index: u32, callback: Callback<u32>) {
	change_endpoint(
		WEB3_ADDR,
		DEFAULT_NETWORKS[index as usize].clone(),
		Callback::new(|_| {}),
	);
}

/// Changes the IPFS endpoint according to the chosen RPC URL.
#[no_mangle]
pub extern "C" fn handle_do_change_endpoint(from: Address, json_cell: Address) {
	// Wait until all characters have been read
	len(
		json_cell,
		Callback::new(move |to_read| {
			// Make a string of n spaces
			let buff = Arc::new(Mutex::new(
				(0..to_read).map(|_| '\0').collect::<Vec<char>>(),
			));
			let n_read = Arc::new(AtomicUsize::new(0));

			// Concurrently read n bytes
			for i in 0..to_read {
				let buff = buff.clone();
				let n_read = n_read.clone();

				read(
					json_cell,
					i,
					Callback::new(move |c: u8| {
						*buff.lock().unwrap().get_mut(i as usize).unwrap() = c as char;

						// We read the last character
						if n_read.fetch_add(1usize, Ordering::SeqCst) == (to_read as usize) - 1usize
						{
							let read_str: String = buff.lock().unwrap().drain(..).collect();
							change_endpoint_ipfs(
								IPFS_ADDR,
								read_str.replace("\"", ""),
								Callback::new(|_| {}),
							);
						}
					}),
				);
			}
		}),
	);
}
