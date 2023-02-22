use beacon_dao_dom::{create_element, eval_js};
use beacon_dao_fetch::{fetch, OptionsBuilder, Response};
use beacon_dao_web3::{change_endpoint, get_endpoint, DEFAULT_NETWORKS};
use serde::{Deserialize, Serialize};
use std::sync::{
	atomic::{AtomicUsize, Ordering},
	Arc, RwLock,
};
use vision_derive::{
	beacon_dao_allocator::{allocate, grow, write},
	with_bindings,
};
use vision_utils::{
	actor::{address, spawn_actor_from},
	types::{
		Address, Callback, ALLOCATOR_ADDR, DOM_ADDR, EXIT_SUCCESS, FETCH_ADDR, PERM_ADDR, WEB3_ADDR,
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
	if from != PERM_ADDR && from != address() {
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

			extern "C" {
				fn print(s: i32);
			}

			let msg = std::ffi::CString::new(format!("ping {}", curr_net_index)).unwrap();

			unsafe {
				print(msg.as_ptr() as i32);
			}

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
								acc, if i == curr_net_index { "; font-weight: bold"} else { "" }, net.name
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

/// Changes the web3 adapter network according to the chosen network.
#[no_mangle]
pub extern "C" fn handle_do_change_network(from: Address, index: u32, callback: Callback<u32>) {
	change_endpoint(
		WEB3_ADDR,
		DEFAULT_NETWORKS[index as usize].clone(),
		Callback::new(|_| {}),
	);
}
