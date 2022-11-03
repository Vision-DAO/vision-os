use serde::{Deserialize, Serialize};
use snafu::{ensure, Snafu};
use vision_derive_internal::with_bindings;
use vision_utils::{
	actor::{address, send_message, spawn_actor},
	types::Address,
};
use wasmer::{FromToNativeWasmType, WasmPtr};

use std::{ffi::CString, sync::RwLock};

macro_rules! eassert {
	($cond:expr) => {
		if !$cond {
			return;
		}
	};
}

macro_rules! assert_isowner {
	($from:ident) => {
		eassert!(OWNER
			.read()
			.ok()
			.map(|owner| *owner == Some($from))
			.unwrap_or(false));
	};
}

macro_rules! is_owner {
	($from:ident) => {
		ensure!(
			*OWNER.read().map_err(|_| Error::MemoryError)? == Some($from),
			NotAllowedSnafu
		);
	};
}

/// The address of an actor representing a memory cell.
pub struct AllocPtr(pub Address);

/// An error encountered while processing an allocator message.
#[derive(Serialize, Deserialize, Debug, Snafu)]
pub enum Error {
	NotAllowed,
	OutOfBounds,
	MemoryError,
}

/// The owner of the memory cell. If this is the manager allocating cells, no
/// owner is specified.
static OWNER: RwLock<Option<Address>> = RwLock::new(None);

/// The contents of the memory cell.
static VAL: RwLock<Vec<u8>> = RwLock::new(Vec::new());

#[with_bindings(self)]
#[no_mangle]
pub extern "C" fn handle_allocate(from: Address, size: u32) -> Result<Address, Error> {
	// Require that we are a manager to allocate memory
	ensure!(
		OWNER
			.read()
			.ok()
			.map(|owner| owner.is_none())
			.unwrap_or(false),
		NotAllowedSnafu
	);

	let memcell = spawn_actor(address());

	// Grow the memory cell by the specified size
	let msg_kind = CString::new("grow").unwrap();
	send_message(
		memcell,
		WasmPtr::from_native(msg_kind.as_ptr() as i32),
		WasmPtr::from_native((&size as *const u32) as i32),
	);

	Ok(memcell)
}

#[cfg(feature = "module")]
#[no_mangle]
pub extern "C" fn init(owner: Address) {
	if let Ok(mut lock) = OWNER.write() {
		lock.replace(owner);
	}
}

#[with_bindings(self)]
#[no_mangle]
pub extern "C" fn handle_read(from: Address, offset: u32) -> Result<u8, Error> {
	is_owner!(from);

	VAL.read()
		.map_err(|_| Error::MemoryError)?
		.get(offset as usize)
		.map(|byte| *byte)
		.ok_or(Error::OutOfBounds)
}

#[with_bindings(self)]
#[no_mangle]
pub extern "C" fn handle_write(from: Address, offset: u32, val: u8) {
	assert_isowner!(from);

	if let Ok(mut lock) = VAL.write() {
		eassert!((offset as usize) < lock.len());

		lock[offset as usize] = val;
	}
}

#[with_bindings(self)]
#[no_mangle]
pub extern "C" fn handle_grow(from: Address, size: u32) {
	assert_isowner!(from);

	if let Ok(mut lock) = VAL.write() {
		// Add `size` zero bytes to the buffer
		for _ in 0..size {
			lock.push(0);
		}
	}
}

#[with_bindings(self)]
#[no_mangle]
pub extern "C" fn handle_shrink(from: Address, size: u32) {
	assert_isowner!(from);

	if let Ok(mut lock) = VAL.write() {
		// Remove `size` bytes from the buffer
		for _ in 0..size {
			lock.pop();
		}
	}
}
