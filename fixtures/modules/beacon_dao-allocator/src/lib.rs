use vision_derive_internal::with_bindings;
use vision_utils::{
	actor::{address, send_message, spawn_actor},
	types::{Address, Callback},
};

use std::{ffi::CString, sync::RwLock};

macro_rules! eassert {
	($cond:expr, $callback:ident) => {
		if !$cond {
			$callback.call(1);

			return;
		}
	};
}

macro_rules! assert_isowner {
	($from:ident, $callback:ident) => {
		eassert!(
			OWNER
				.read()
				.ok()
				.map(|owner| *owner == Some($from))
				.unwrap_or(false),
			$callback
		);
	};
}

/// The owner of the memory cell. If this is the manager allocating cells, no
/// owner is specified.
static OWNER: RwLock<Option<Address>> = RwLock::new(None);

/// The contents of the memory cell.
static VAL: RwLock<Vec<u8>> = RwLock::new(Vec::new());

#[cfg(feature = "module")]
#[no_mangle]
pub extern "C" fn init(owner: Address) {
	if let Ok(mut lock) = OWNER.write() {
		lock.replace(owner);
	}
}

#[no_mangle]
#[with_bindings(self)]
pub extern "C" fn handle_allocate(from: Address, callback: Callback<Address>) {
	// Require that we are a manager to allocate memory
	eassert!(
		OWNER
			.read()
			.ok()
			.map(|owner| owner.is_none())
			.unwrap_or(false),
		callback
	);

	let cell = spawn_actor(address());

	reassign_priv(
		cell,
		from,
		Callback::new(move |_| {
			callback.call(cell);
		}),
	);
}

/// Reassigns the owner of the memory cell.
#[no_mangle]
#[with_bindings(self)]
pub extern "C" fn handle_reassign(from: Address, new_owner: Address, callback: Callback<u8>) {
	assert_isowner!(from, callback);

	if let Ok(mut lock) = OWNER.write() {
		let old = lock.clone();
		lock.replace(new_owner);

		callback.call(0);
	} else {
		callback.call(1);
	}
}

#[no_mangle]
#[with_bindings(self)]
pub extern "C" fn handle_read(from: Address, offset: u32, callback: Callback<u8>) {
	callback.call(
		VAL.read()
			.unwrap()
			.get(offset as usize)
			.map(|byte| *byte)
			.unwrap(),
	);
}

/// Synchronously reads from the cell.
#[no_mangle]
pub extern "C" fn read_sync(offset: u32) -> u8 {
	VAL.read()
		.unwrap()
		.get(offset as usize)
		.map(|byte| *byte)
		.unwrap()
}

/// Synchronously gets the length of the cell.
#[no_mangle]
pub extern "C" fn len_sync() -> u32 {
	VAL.read().unwrap().len().try_into().unwrap()
}

#[no_mangle]
#[with_bindings(self)]
pub extern "C" fn handle_write(from: Address, offset: u32, val: u8, callback: Callback<u8>) {
	assert_isowner!(from, callback);

	if let Ok(mut lock) = VAL.write() {
		eassert!((offset as usize) < lock.len(), callback);

		lock[offset as usize] = val;
	}

	callback.call(0);
}

#[no_mangle]
#[with_bindings(self)]
pub extern "C" fn handle_grow(from: Address, size: u32, callback: Callback<u8>) {
	assert_isowner!(from, callback);

	if let Ok(mut lock) = VAL.write() {
		// Add `size` zero bytes to the buffer
		for _ in 0..size {
			lock.push(0);
		}
	}

	callback.call(0);
}

#[no_mangle]
#[with_bindings(self)]
pub extern "C" fn handle_len(from: Address, callback: Callback<u32>) {
	callback.call(VAL.read().unwrap().len().try_into().unwrap());
}
