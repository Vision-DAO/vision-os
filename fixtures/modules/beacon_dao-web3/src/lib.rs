use beacon_dao_fetch::{fetch, Method, Options, OptionsBuilder};
use beacon_dao_permissions::{has_permission, register_permission};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
	error::Error as StdError,
	sync::{Arc, RwLock},
};
use vision_derive::with_bindings;
use vision_utils::types::{Address, Callback, DISPLAY_MANAGER_ADDR, FETCH_ADDR, PERM_ADDR};

const PERM_CHANGE: &'static str = "change network";
const PERM_CHANGE_DESC: &'static str = "change which Ethereum network you're connected to.";

const PERM_USE: &'static str = "use web3";
const PERM_USE_DESC: &'static str = "interact with the Ethereum network.";

/// Errors that might be encountered when using this API.
#[derive(Serialize, Deserialize)]
pub enum Error {
	NoPermission,
	SerializationError,
}

/// An EVM compatible network.
#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Network {
	pub chain_id: usize,
	pub name: String,
	pub ticker: String,
	pub rpc_url: String,
}

/// Networks known by the web3 adapter by default.
lazy_static::lazy_static! {
	pub static ref DEFAULT_NETWORKS: [Network; 3] = [
		Network {
			chain_id: 1,
			name: String::from("Ethereum"),
			ticker: String::from("ETH"),
			rpc_url: String::from("https://eth-rpc.gateway.pokt.network"),
		},
		Network {
			chain_id: 42161,
			name: String::from("Arbitrum One"),
			ticker: String::from("ETH"),
			rpc_url: String::from("https://endpoints.omniatech.io/v1/arbitrum/one/public"),
		},
		Network {
			chain_id: 421613,
			name: String::from("Arbitrum Goerli"),
			ticker: String::from("AGOR"),
			rpc_url: String::from("https://endpoints.omniatech.io/v1/arbitrum/goerli/public"),
		},
	];
}

/// The current RPC endpoint in use.
lazy_static::lazy_static! {
	pub static ref ADAPTER_NET: RwLock<Network> = RwLock::new(DEFAULT_NETWORKS[0].clone());
}

#[cfg(feature = "module")]
#[no_mangle]
pub extern "C" fn handle_init_async(owner: Address) {
	register_permission(
		PERM_ADDR,
		PERM_CHANGE.to_owned(),
		PERM_CHANGE_DESC.to_owned(),
		Callback::new(|_| {}),
	);

	register_permission(
		PERM_ADDR,
		PERM_USE.to_owned(),
		PERM_USE_DESC.to_owned(),
		Callback::new(|_| {}),
	);
}

/// Replaces the RPC endpoint in use with the new one, if the user has permissions to do so.
#[no_mangle]
#[with_bindings]
pub extern "C" fn handle_change_endpoint(
	from: Address,
	new_net: Network,
	callback: Callback<Result<(), Error>>,
) {
	has_permission(
		PERM_ADDR,
		from,
		PERM_CHANGE.to_owned(),
		Callback::new(move |has_permission: bool| {
			if !has_permission && from != DISPLAY_MANAGER_ADDR {
				callback.call(Err(Error::NoPermission));

				return;
			}

			// Replace the provider URL
			let mut l = ADAPTER_NET.write().unwrap();
			*l = new_net;

			callback.call(Ok(()));
		}),
	)
}

/// Gets the Network currently connected to by this web3 adapter.
#[no_mangle]
#[with_bindings]
pub extern "C" fn handle_get_endpoint(from: Address, callback: Callback<Network>) {
	callback.call(ADAPTER_NET.write().unwrap().clone());
}

/// A request to a JSON RPC API
#[derive(Serialize, Deserialize)]
pub struct Request {
	method: String,
	jsonrpc: String,
	params: Vec<Value>,
	id: usize,
}

#[derive(Serialize, Deserialize)]
pub enum BlockSelector {
	Latest,
	Earliest,
	Pending,
	BlockNumber(usize),
}

impl From<BlockSelector> for String {
	fn from(s: BlockSelector) -> Self {
		match s {
			BlockSelector::Latest => String::from("latest"),
			BlockSelector::Earliest => String::from("earliest"),
			BlockSelector::Pending => String::from("pending"),
			BlockSelector::BlockNumber(n) => format!("{:x}", n),
		}
	}
}

/// Parameters for a request to make a call to an Ethereum contract.
#[derive(Serialize, Deserialize)]
pub struct TransactionCall {
	pub from: Option<[u8; 20]>,
	pub to: [u8; 20],
	pub gas: Option<usize>,
	pub gasPrice: Option<usize>,
	pub value: Option<usize>,
	pub data: Option<String>,
}

/// Executes a call on an Ethereum contract. Wraps eth_call.
#[no_mangle]
#[with_bindings]
pub fn handle_eth_call(
	from: Address,
	params_0: TransactionCall,
	params_1: BlockSelector,
	callback: Callback<Result<String, Error>>,
) {
	has_permission(
		PERM_ADDR,
		from,
		PERM_USE.to_owned(),
		Callback::new(move |has_permission: bool| {
			if !has_permission && from != DISPLAY_MANAGER_ADDR {
				callback.call(Err(Error::NoPermission));

				return;
			}

			let url = if let Ok(lock) = ADAPTER_NET.read() {
				lock.rpc_url.to_owned()
			} else {
				callback.call(Err(Error::NoPermission));

				return;
			};

			let (p1, p2) = match serde_json::to_value(params_0)
				.and_then(|p1| serde_json::to_value(params_1).map(|p2| (p1, p2)))
			{
				Ok((p1, p2)) => (p1, p2),
				Err(_) => {
					callback.call(Err(Error::SerializationError));
					return;
				}
			};

			// Use the fetch client to make the request
			fetch(
				FETCH_ADDR,
				url,
				OptionsBuilder {
					method: Some(Method::POST),
					headers: None,
					body: Some(Request {
						method: String::from("eth_call"),
						jsonrpc: String::from("2.0"),
						params: vec![p1, p2],
						id: 1,
					}),
				}
				.into(),
				Callback::new(|resp| {
					extern "C" {
						fn print(s: i32);
					}

					let msg = std::ffi::CString::new(format!("resp {:?}", resp)).unwrap();

					unsafe {
						print(msg.as_ptr() as i32);
					}
				}),
			);
		}),
	)
}
