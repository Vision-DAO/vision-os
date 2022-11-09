use serde::{Deserialize, Serialize};
use snafu::{ensure, Snafu};
use vision_derive_internal::with_bindings;
use vision_utils::{
	actor::{address, send_message, spawn_actor},
	types::Address,
};

use std::{ffi::CString, ptr, sync::RwLock};

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

extern "C" {
	fn print(s: i32);
}

#[no_mangle]
pub extern "C" fn handle_allocate(from: Address, size: u32) -> Result<Address, Error> {
	unsafe {
		if let Ok(msg) = CString::new("handling allocate") {
			print(msg.as_ptr() as i32);
		}
	};

	// Require that we are a manager to allocate memory
	ensure!(
		OWNER
			.read()
			.ok()
			.map(|owner| owner.is_none())
			.unwrap_or(false),
		NotAllowedSnafu
	);

	unsafe {
		if let Ok(msg) = CString::new("79") {
			print(msg.as_ptr() as i32);
		}
	};

	let memcell = spawn_actor(address());

	unsafe {
		if let Ok(msg) = CString::new("spawned child") {
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

	Ok(memcell)
}

/* Manually-generated ABI code for bootstrapping macro */

pub static PIPELINE_ALLOCATE: RwLock<Option<Result<Address, Error>>> = RwLock::new(None);

#[macro_export]
macro_rules! use_allocate {
	() => {
		#[no_mangle]
		pub extern "C" fn handle_allocate(from: Address, arg: Address) {

			PIPELINE_ALLOCATE.
		}
	}
}

#[cfg(feature = "module")]
#[no_mangle]
pub extern "C" fn allocate(to: Address, size: u32) -> Option<Result<Address, Error>> {
	let msg_kind = CString::new("allocate").unwrap();

	send_message(to, msg_kind.as_ptr() as i32, ptr::addr_of(size) as i32);
	PIPELINE_ALLOCATE.write().unwrap().take()
}

/* End manual ABI */

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
