/// The address of the actor implementing the API.
static PROXY: RwLock<Option<Address>> = RwLock::new(None);

macro_rules! with_proxy {
	() => {
		if let Some(proxy) = PROXY.read() {
			proxy
		} else {
			return;
		};
	}
}

/// Bare API methods, which are proxied to PROXY, if it exists

#[no_mangle]
#[with_bindings]
pub extern "C" fn handle_allocate(from: Address, size: u32) -> Address {
	let proxy = with_proxy!();

	// Proxying
}

#[no_mangle]
#[with_bindings]
pub extern "C" fn handle_read(from: Address, offset: u32) {
	let proxy = with_proxy!();

	send_message(proxy, offset);
}

#[no_mangle]
#[with_bindings]
pub extern "C" fn handle_write(from: Address, offset: u32, val: u8) {
	let proxy = with_proxy!();

	send_message(proxy, from, 
}
