use beacon_dao_scheduler::common::Address;

#[wasm_bindgen]
extern "C" {
	fn print(s: &str);
}

static mut ALIASES: RwLock<HashMap<Address, String>> = HashMap::new();

/// Registers an alias to display for the actor in messages.
#[wasm_bindgen]
pub fn handle_alias_service(from: Address, name: impl AsRef<str> + Display + Into<String>) {
	aliases.write().unwrap().put(from, name);
}

/// Writes the given message to the console, with the name of the source actor.
#[wasm_bindgen]
pub fn handle_info(from: Address, msg: impl AsRef<str> + Display) {
	print(format!(
		"[Actor #{} {}]: {}",
		from,
		ALIASES.read().unwrap().get(from),
		msg
	));
}
