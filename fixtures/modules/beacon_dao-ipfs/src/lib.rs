use beacon_dao_fetch::{fetch, Method, Options, OptionsBuilder, Response};
use beacon_dao_permissions::{has_permission, register_permission};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
	collections::HashMap,
	error::Error as StdError,
	sync::{Arc, RwLock},
};
use vision_derive::with_bindings;
use vision_utils::types::{Address, Callback, DISPLAY_MANAGER_ADDR, FETCH_ADDR, PERM_ADDR};

const PERM_CHANGE: &'static str = "change provider";
const PERM_CHANGE_DESC: &'static str = "change which IPFS provider you're connected to.";

const PERM_USE: &'static str = "use IPFS";
const PERM_USE_DESC: &'static str = "interact with the IPFS network.";

/// Errors that might be encountered when using this API.
#[derive(Serialize, Deserialize)]
pub enum Error {
	NoPermission,
	SerializationError,
	ServerError,
}

/// Providers known by the IPFS adapter by default.
const DEFAULT_PROVIDER: &'static str = "https://dweb.link/";

/// The current RPC endpoint in use.
lazy_static::lazy_static! {
	pub static ref ADAPTER_RPC: RwLock<String> = RwLock::new(DEFAULT_PROVIDER.to_owned());
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
pub extern "C" fn handle_change_rpc_endpoint(
	from: Address,
	new_rpc: String,
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
			let mut l = ADAPTER_RPC.write().unwrap();
			*l = new_rpc;

			callback.call(Ok(()));
		}),
	)
}

/// Gets the Network currently connected to by this web3 adapter.
#[no_mangle]
#[with_bindings]
pub extern "C" fn handle_get_rpc_endpoint(from: Address, callback: Callback<String>) {
	callback.call(ADAPTER_RPC.write().unwrap().clone());
}
