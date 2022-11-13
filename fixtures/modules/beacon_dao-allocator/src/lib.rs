use vision_derive_internal::with_bindings;
use vision_utils::{
	actor::{address, send_message, spawn_actor},
	types::Address,
};

use std::{ffi::CString, sync::RwLock};

macro_rules! eassert {
	($cond:expr, $msg:expr) => {
		if !$cond {
			panic!($msg)
		}
	};
}

macro_rules! assert_isowner {
	($from:ident) => {
		eassert!(
			OWNER
				.read()
				.ok()
				.map(|owner| *owner == Some($from))
				.unwrap_or(false),
			"Insufficient permissions."
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

extern "C" {
	fn print(s: i32);
}

#[no_mangle]
#[with_bindings(self)]
pub extern "C" fn handle_allocate(from: Address, size: u32) -> Address {
	unsafe {
		if let Ok(msg) = CString::new("handling allocate") {
			print(msg.as_ptr() as i32);
		}
	};

	// Require that we are a manager to allocate memory
	eassert!(
		OWNER
			.read()
			.ok()
			.map(|owner| owner.is_none())
			.unwrap_or(false),
		"Cannot allocate from non-root cell."
	);

	unsafe {
		if let Ok(msg) = CString::new("79") {
			print(msg.as_ptr() as i32);
		}
	};

	let memcell = spawn_actor(address());

	unsafe {
		if let Ok(msg) = CString::new(format!("spawned child {}", memcell)) {
			print(msg.as_ptr() as i32);
		}
	};

	// Grow the memory cell by the specified size
	let msg_kind = CString::new("grow").unwrap();
	send_message(
		memcell,
		msg_kind.as_ptr() as i32,
		(&size as *const u32) as i32,
	);

	memcell
}

#[no_mangle]
#[with_bindings(self)]
pub extern "C" fn handle_read(from: Address, offset: u32) -> u8 {
	VAL.read()
		.unwrap()
		.get(offset as usize)
		.map(|byte| *byte)
		.unwrap()
}

#[no_mangle]
#[with_bindings(self)]
pub extern "C" fn handle_write(from: Address, offset: u32, val: u8) {
	assert_isowner!(from);

	if let Ok(mut lock) = VAL.write() {
		eassert!((offset as usize) < lock.len(), "Out of bounds.");

		lock[offset as usize] = val;
	}
}

#[no_mangle]
#[with_bindings(self)]
pub extern "C" fn handle_grow(from: Address, size: u32) {
	assert_isowner!(from);

	if let Ok(mut lock) = VAL.write() {
		// Add `size` zero bytes to the buffer
		for _ in 0..size {
			lock.push(0);
		}
	}
}

#[no_mangle]
#[with_bindings(self)]
pub extern "C" fn handle_len(from: Address) -> u32 {
	VAL.read().unwrap().len()
}
