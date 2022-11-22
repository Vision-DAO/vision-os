use std::{collections::HashMap, ffi::CString, ptr, sync::RwLock};

use once_cell::sync::Lazy;
use vision_derive::with_bindings;
use vision_utils::types::{Address, Callback};

static ALIASES: Lazy<RwLock<HashMap<Address, String>>> = Lazy::new(|| RwLock::new(HashMap::new()));

/// Registers an alias to display for the actor in messages.
#[with_bindings]
#[no_mangle]
pub extern "C" fn handle_alias_service(from: Address, name: String, callback: Callback<u8>) {
	if let Some(mut lock) = ALIASES.write().ok() {
		lock.insert(from, name);
		callback.call(0);
	} else {
		callback.call(1);
	}
}

/// Writes the given message to the console, with the name of the source actor.
#[with_bindings]
#[no_mangle]
pub extern "C" fn handle_info(from: Address, msg: String, callback: Callback<u8>) {
	if let Some(_) = inner_info(from, msg) {
		callback.call(0);
	} else {
		callback.call(1);
	}
}

fn inner_info(from: Address, msg: String) -> Option<()> {
	extern "C" {
		fn print(s: i32);
	}

	let msg = CString::new(format!(
		"INFO [Actor #{}{}]: {}",
		from,
		ALIASES
			.read()
			.ok()?
			.get(&from)
			.map(|alias| format!(" {alias}"))
			.unwrap_or_default(),
		msg
	))
	.ok()?;

	unsafe {
		print(msg.as_ptr() as i32);
	}

	Some(())
}
