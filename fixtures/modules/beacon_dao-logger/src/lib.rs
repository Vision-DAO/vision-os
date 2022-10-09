use std::{collections::HashMap, fmt::Display, sync::RwLock};

use vision_utils::types::Address;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
extern "C" {
	fn print(s: &str);
}

static mut ALIASES: RwLock<HashMap<Address, String>> = RwLock::new(HashMap::new());

/// Registers an alias to display for the actor in messages.
#[wasm_bindgen]
pub fn handle_alias_service(from: Address, name: Address) {
	ALIASES.write().unwrap().insert(from, name.to_owned());
}

/// Writes the given message to the console, with the name of the source actor.
#[wasm_bindgen]
pub fn handle_info(from: Address, msg: Address) {
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
