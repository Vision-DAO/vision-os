use beacon_dao_allocator::{allocate, read, write};
use beacon_dao_permissions::{has_permission, register_permission};
use vision_utils::types::{Callback, ALLOCATOR_IMPL_ADDR, PERM_ADDR};

const PERM: &'static str = "change_proxy_allocator";
const DESCRIPTION: &'static str =
	"Allows the app to change which app your Vision OS uses for allocation.";

/// The address of the actor implementing the API.
static PROXY: RwLock<Option<Address>> = RwLock::new(Some(ALLOCATOR_IMPL_ADDR));

macro_rules! with_proxy {
	() => {
		if let Some(proxy) = PROXY.read() {
			proxy
		} else {
			return;
		};
	};
}

#[cfg(feature = "module")]
#[no_mangle]
pub fn init(owner: Address) {
	register_permission(PERM.into_owned(), DESCRIPTION.into_owned());
}

/// Bare API methods, which are proxied to PROXY, if it exists

#[no_mangle]
#[with_bindings]
pub extern "C" fn handle_change_proxy(from: Address, proxy: Address, callback: Callback<u8>) {
	has_permission(
		PERM_ADDR,
		from,
		"change_proxy_allocator".into(),
		Callback::new(|has_perm| {
			if !has_perm {
				callback(1);

				return;
			}

			if let Some(ref mut proxy) = PROXY.write() {
				proxy.replace(Some(proxy));

				callback(0);
			} else {
				callback(1);
			}
		}),
	);
}

#[no_mangle]
#[with_bindings]
pub extern "C" fn handle_allocate(from: Address, size: u32, callback: Callback<Address>) {
	let proxy = with_proxy!();

	allocate(proxy, from, size, Callback::new(|addr| callback(addr)));
}
