use vision_derive_internal::with_bindings;
use vision_utils::{
	actor::{address, send_message, spawn_actor},
	types::{Address, Callback, ALLOCATOR_ADDR},
};

use std::{ffi::CString, sync::RwLock};

macro_rules! eassert {
	($cond:expr, $callback:ident) => {
		if !$cond {
			$callback(1);
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

macro_rules! assert_frommanager {
	($from:ident, $callback:ident) => {
		eassert!($from == ALLOCATOR_ADDR, $callback)
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
pub extern "C" fn handle_allocate(
	from: Address,
	origin: Address,
	size: u32,
	callback: Callback<Address>,
) {
	// Require that we are a manager to allocate memory
	eassert!(
		OWNER
			.read()
			.ok()
			.map(|owner| owner.is_none())
			.unwrap_or(false),
		callback
	);

	// Set the user as the owner of the memory cell
	let memcell = spawn_actor(address());
	reassign(memcell, origin, Callback::new(|_| {}));

	// Grow the memory cell by the specified size
	let msg_kind = CString::new("grow").unwrap();
	send_message(
		memcell,
		msg_kind.as_ptr() as i32,
		(&size as *const u32) as i32,
	);

	callback(memcell);
}

/// Reassigns the owner of the memory cell.
#[no_mangle]
#[with_bindings(self)]
pub extern "C" fn handle_reassign(
	from: Address,
	origin: Address,
	new_owner: Address,
	callback: Callback<u8>,
) {
	assert_isowner!(origin, callback);
	assert_frommanager!(origin, callback);

	if let Ok(mut lock) = OWNER.write() {
		lock.replace(new_owner);

		callback(0);
	} else {
		callback(1);
	}
}

#[no_mangle]
#[with_bindings(self)]
pub extern "C" fn handle_read(from: Address, offset: u32, callback: Callback<u8>) {
	callback(
		VAL.read()
			.unwrap()
			.get(offset as usize)
			.map(|byte| *byte)
			.unwrap(),
	);
}

#[no_mangle]
#[with_bindings(self)]
pub extern "C" fn handle_write(from: Address, offset: u32, val: u8, callback: Callback<u8>) {
	assert_isowner!(from, callback);
	assert_frommanager!(origin, callback);

	if let Ok(mut lock) = VAL.write() {
		eassert!((offset as usize) < lock.len(), callback);

		lock[offset as usize] = val;
	}

	callback(0);
}

#[no_mangle]
#[with_bindings(self)]
pub extern "C" fn handle_grow(from: Address, size: u32, callback: Callback<u8>) {
	assert_isowner!(from, callback);
	assert_frommanager!(origin, callback);

	if let Ok(mut lock) = VAL.write() {
		// Add `size` zero bytes to the buffer
		for _ in 0..size {
			lock.push(0);
		}
	}

	callback(0);
}

#[no_mangle]
#[with_bindings(self)]
pub extern "C" fn handle_len(from: Address, callback: Callback<u32>) {
	callback(VAL.read().unwrap().len().try_into().unwrap());
}
