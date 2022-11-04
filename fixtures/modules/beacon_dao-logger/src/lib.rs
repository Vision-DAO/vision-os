use std::{collections::HashMap, sync::RwLock};

use once_cell::sync::Lazy;
use vision_derive::with_bindings;
use vision_utils::types::Address;

static ALIASES: Lazy<RwLock<HashMap<Address, String>>> = Lazy::new(|| RwLock::new(HashMap::new()));

/// Registers an alias to display for the actor in messages.
#[with_bindings]
#[no_mangle]
pub extern "C" fn handle_alias_service(from: Address, name: String) {
	ALIASES.write().unwrap().insert(from, name);
}

/// Writes the given message to the console, with the name of the source actor.
#[with_bindings]
#[no_mangle]
pub extern "C" fn handle_info(from: Address, msg: String) {
	extern "C" {
		fn print(s: &str);
	}

	unsafe {
		print(&format!(
			"INFO [Actor #{}{}]: {}",
			from,
			ALIASES
				.read()
				.unwrap()
				.get(&from)
				.map(|alias| format!(" {alias}"))
				.unwrap_or_default(),
			msg
		));
	}
}
