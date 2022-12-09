use beacon_dao_permissions::{has_permission, register_permission};
use vision_derive::with_bindings;
use vision_utils::types::{Address, Callback, PERM_ADDR};

const PERM: &'static str = "control your computer";
const DESCRIPTION: &'static str =
	"change what's on your display, your preferences, and your password.";

#[cfg(feature = "module")]
#[no_mangle]
pub extern "C" fn init(owner: Address) {
	register_permission(
		PERM_ADDR,
		PERM.to_owned(),
		DESCRIPTION.to_owned(),
		Callback::new(|_| {}),
	);
}

/// Appends a new element to the DOM, returning 0 if successful, and 1 if unsuccessful,
/// if the caller has permission to do so.
#[no_mangle]
#[with_bindings]
pub extern "C" fn handle_create_element(
	from: Address,
	kind: String,
	src: String,
	callback: Callback<u8>,
) {
	has_permission(
		PERM_ADDR,
		from,
		PERM.to_owned(),
		Callback::new(|has_permission| {
			extern "C" {
				fn append_element(k: i32, s: i32) -> u8;
			}

			let kind = std::ffi::CString::new(kind).unwrap();
			let elem = std::ffi::CString::new(src).unwrap();

			unsafe {
				callback.call(append_element(kind.as_ptr() as i32, elem.as_ptr() as i32));
			}
		}),
	);
}
