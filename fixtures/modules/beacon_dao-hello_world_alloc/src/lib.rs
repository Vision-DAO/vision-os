use beacon_dao_logger::{alias_service, info};
use vision_derive::with_bindings;
use vision_utils::types::Address;

#[cfg(feature = "module")]
#[no_mangle]
pub extern "C" fn init(parent: Address) {
	alias_service(3, "Test Actor".to_owned());
}

#[no_mangle]
pub extern "C" fn handle_test(from: Address) {
	info(3, "Bro".to_owned());
}
