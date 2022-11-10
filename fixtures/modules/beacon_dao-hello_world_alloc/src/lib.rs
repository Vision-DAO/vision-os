use beacon_dao_logger::{alias_service, info, use_alias_service, use_info};
use vision_derive::with_bindings;
use vision_utils::types::Address;

use_info!();
use_alias_service!();

#[cfg(feature = "module")]
#[no_mangle]
pub extern "C" fn init(parent: Address) {
	alias_service(3, "Test Actor".to_owned());
}

#[no_mangle]
pub extern "C" fn handle_test(from: Address) {
	info(3, "Bro".to_owned());
}
