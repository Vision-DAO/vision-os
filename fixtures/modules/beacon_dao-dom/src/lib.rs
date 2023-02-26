pub use beacon_dao_permissions;
use beacon_dao_permissions::{has_permission, register_permission};
use vision_derive::with_bindings;
use vision_utils::types::{Address, Callback, DISPLAY_MANAGER_ADDR, FETCH_ADDR, PERM_ADDR};

const PERM: &'static str = "control your computer";
const DESCRIPTION: &'static str =
	"change what's on your display, your preferences, and your password.";

#[cfg(feature = "module")]
#[no_mangle]
pub extern "C" fn handle_init_async(owner: Address) {
	register_permission(
		PERM_ADDR,
		PERM.to_owned(),
		DESCRIPTION.to_owned(),
		Callback::new(|_| {}),
	);
}

/// Returns Ok(()) if the element was created successfully, or Err otherwise.
fn do_create_element(
	has_permission: bool,
	from: Address,
	kind: String,
	src: String,
) -> Result<u8, ()> {
	if !has_permission && from != DISPLAY_MANAGER_ADDR && from != FETCH_ADDR {
		return Err(());
	}

	extern "C" {
		fn append_element(k: i32, s: i32) -> u8;
	}

	let kind = std::ffi::CString::new(kind).map_err(|_| ())?;
	let elem = std::ffi::CString::new(src).map_err(|_| ())?;

	unsafe { Ok(append_element(kind.as_ptr() as i32, elem.as_ptr() as i32)) }
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
		Callback::new(move |has_permission: bool| {
			if let Ok(stat) = do_create_element(has_permission, from, kind, src) {
				callback.call(stat);
			} else {
				callback.call(1);
			}
		}),
	);
}

/// Returns Ok(()) if the JS was evaluated successfully.
fn do_eval_js(has_permission: bool, from: Address, src: String) -> Result<u8, ()> {
	if !has_permission && from != DISPLAY_MANAGER_ADDR && from != FETCH_ADDR {
		return Err(());
	}

	extern "C" {
		fn eval_js(s: i32) -> u8;
	}

	let src = std::ffi::CString::new(format!(
		"{{
												let impulse = (to, msgName, ...args) => {{
													window.impulse({from}, to, msgName, args);
												}}

												let address = () => {from};

												{src}
											  }}"
	))
	.map_err(|_| ())?;

	unsafe { Ok(eval_js(src.as_ptr() as i32)) }
}

/// Executes arbitrary JS if the user has permissions to do so.
#[no_mangle]
#[with_bindings]
pub extern "C" fn handle_eval_js(from: Address, src: String, callback: Callback<u8>) {
	has_permission(
		PERM_ADDR,
		from,
		PERM.to_owned(),
		Callback::new(move |has_permission: bool| {
			if let Ok(stat) = do_eval_js(has_permission, from, src) {
				callback.call(stat);
			} else {
				callback.call(1);
			}
		}),
	);
}
