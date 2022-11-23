use beacon_dao_permissions::{has_permission, register_permission};
use vision_derive_internal::with_bindings;
use vision_utils::types::{Address, Callback, ALLOCATOR_IMPL_ADDR, PERM_ADDR};

use std::{ops::DerefMut, sync::RwLock};

const PERM: &'static str = "change_proxy_allocator";
const DESCRIPTION: &'static str =
	"Allows the app to change which app your Vision OS uses for allocation.";

/// The address of the actor implementing the API.
static PROXY: RwLock<Option<Address>> = RwLock::new(Some(ALLOCATOR_IMPL_ADDR));

macro_rules! with_proxy {
	() => {
		if let Ok(Some(proxy)) = PROXY.read().map(|cts| *cts) {
			proxy
		} else {
			return;
		}
	};
}

#[cfg(feature = "module")]
#[no_mangle]
pub fn init(owner: Address) {
	register_permission(
		PERM_ADDR,
		PERM.to_owned(),
		DESCRIPTION.to_owned(),
		Callback::new(|_| {}),
	);
}

/// Bare API methods, which are proxied to PROXY, if it exists

#[no_mangle]
#[with_bindings(self)]
pub extern "C" fn handle_change_proxy(from: Address, proxy: Address, callback: Callback<u8>) {
	has_permission(
		PERM_ADDR,
		from,
		"change_proxy_allocator".into(),
		Callback::new(move |has_perm: bool| {
			if !has_perm {
				callback.call(1);

				return;
			}

			if let Ok(ref mut proxy_buf) = PROXY.write() {
				proxy_buf.replace(proxy);

				callback.call(0);
			} else {
				callback.call(1);
			}
		}),
	);
}

#[no_mangle]
#[with_bindings(self)]
pub extern "C" fn handle_allocate(from: Address, size: u32, callback: Callback<Address>) {
	extern "C" {
		fn print(s: i32);
	}

	unsafe {
		let msg = std::ffi::CString::new("allocating on the heap").unwrap();
		print(msg.as_ptr() as i32);
	}

	let proxy = with_proxy!();

	beacon_dao_allocator::allocate(proxy, from, size, Callback::new(|addr| callback.call(addr)));
}
