use beacon_dao_logger_manager::info;
use beacon_dao_permissions_consent::request_permission;
use vision_utils::types::{Address, Callback, LOGGER_ADDR, PERM_AGENT_ADDR};

#[cfg(feature = "module")]
#[no_mangle]
pub fn handle_init_async(from: Address) {
	request_permission(
		PERM_AGENT_ADDR,
		String::from("make_http_request"),
		Callback::new(|acquired| {
			info(LOGGER_ADDR, format!("{}", acquired), Callback::new(|_| {}));
		}),
	);
}
