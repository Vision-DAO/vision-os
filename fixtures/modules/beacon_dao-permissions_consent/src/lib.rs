use beacon_dao_display_manager::{system_dialogue, DialogueKind};
use beacon_dao_logger_manager::info;
use beacon_dao_permissions::{get_permission, set_permission};
use vision_derive::with_bindings;
use vision_utils::types::{Address, Callback, DISPLAY_MANAGER_ADDR, LOGGER_ADDR, PERM_ADDR};

#[no_mangle]
#[with_bindings]
pub fn handle_request_permission(from: Address, permission: String, callback: Callback<bool>) {
	// Send the user a prompt asking them for permission to do x thing, then
	// update the actor's permissions accordingly
	get_permission(
		PERM_ADDR,
		permission.clone(),
		Callback::new(move |desc: Option<String>| {
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
