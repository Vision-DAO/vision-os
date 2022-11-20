use beacon_dao_allocator::{allocate, read, write};
use vision_utils::types::Callback;

/// The address of the actor implementing the API.
static PROXY: RwLock<Option<Address>> = RwLock::new(None);

macro_rules! with_proxy {
	() => {
		if let Some(proxy) = PROXY.read() {
			proxy
		} else {
			return;
		};
	};
}

/// Bare API methods, which are proxied to PROXY, if it exists

#[no_mangle]
#[with_bindings]
pub extern "C" fn handle_allocate(from: Address, size: u32, callback: Callback<Address>) {
	let proxy = with_proxy!();

	allocate(proxy, from, size, Callback::new(|addr| callback(addr)));
}
