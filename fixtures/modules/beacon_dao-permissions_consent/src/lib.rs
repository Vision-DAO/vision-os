use beacon_dao_display_manager::{system_dialogue, DialogueKind};
use beacon_dao_permissions::{get_permission, set_permission};
use vision_derive::with_bindings;
use vision_utils::types::{Address, Callback, DISPLAY_MANAGER_ADDR, PERM_ADDR, LOGGER_ADDR};
use beacon_dao_logger_manager::info;

#[no_mangle]
#[with_bindings]
pub fn handle_request_permission(from: Address, permission: String, callback: Callback<bool>) {
	info(LOGGER_ADDR, format!("perm request: {}", permission), Callback::new(|_| {}));

	// Send the user a prompt asking them for permission to do x thing, then
	// update the actor's permissions accordingly
	get_permission(
		PERM_ADDR,
		permission.clone(),
		Callback::new(move |desc: Option<String>| {
			info(LOGGER_ADDR, format!("perm: {:?}", desc), Callback::new(|_| {}));

			let desc = if let Some(desc) = desc {
				desc
			} else {
				callback.call(false);

				return;
			};

			system_dialogue(
				DISPLAY_MANAGER_ADDR,
				format!("Grant Actor #{} Permission to {}", from, permission),
				desc,
				DialogueKind::Choice(String::from("No"), String::from("Yes")),
				Callback::new(move |stat| {
					if stat == 1 {
						set_permission(
							PERM_ADDR,
							from,
							permission,
							Callback::new(|_| callback.call(true)),
						);
					} else {
						callback.call(false);
					}
				}),
			);
		}),
	);
}
