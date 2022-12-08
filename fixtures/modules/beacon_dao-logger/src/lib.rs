use std::{collections::HashMap, ffi::CString, ptr, sync::RwLock};

use once_cell::sync::Lazy;
use vision_derive::with_bindings;
use vision_utils::types::{Address, Callback, LOGGER_ADDR};

static ALIASES: Lazy<RwLock<HashMap<Address, String>>> = Lazy::new(|| RwLock::new(HashMap::new()));

macro_rules! eassert {
	($cond:expr, $cb:ident) => {
		if (!$cond) {
			$cb.call(1);

			return;
		}
	};
}

/// Registers an alias to display for the actor in messages.
#[with_bindings]
#[no_mangle]
pub extern "C" fn handle_alias_service(
	from: Address,
	origin: Address,
	name: String,
	callback: Callback<u8>,
) {
	// Ensure that the call is coming from the manager that proxies requests
	// This pattern is very common.
	eassert!(from == LOGGER_ADDR, callback);

	if let Some(mut lock) = ALIASES.write().ok() {
		lock.insert(origin, name);
		callback.call(0);
	} else {
		callback.call(1);
	}
}

/// Writes the given message to the console, with the name of the source actor.
#[with_bindings]
#[no_mangle]
pub extern "C" fn handle_info(from: Address, origin: Address, msg: String, callback: Callback<u8>) {
	eassert!(from == LOGGER_ADDR, callback);

	if let Some(_) = inner_info(origin, msg) {
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
