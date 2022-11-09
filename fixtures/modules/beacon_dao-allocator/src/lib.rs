use serde::{Deserialize, Serialize};
use snafu::Snafu;
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
pub extern "C" fn handle_allocate(from: Address, size: u32) {
	unsafe {
		if let Ok(msg) = CString::new("handling allocate") {
			print(msg.as_ptr() as i32);
		}
	};

	// Require that we are a manager to allocate memory
	eassert!(OWNER
		.read()
		.ok()
		.map(|owner| owner.is_none())
		.unwrap_or(false));

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

	let msg_kind = CString::new("allocate").unwrap();
	send_message(
		from,
		msg_kind.as_ptr() as i32,
		ptr::addr_of!(memcell) as i32,
	);
}

/* Manually-generated ABI code for bootstrapping macro */

pub static PIPELINE_ALLOCATE: RwLock<Option<Address>> = RwLock::new(None);

#[macro_export]
macro_rules! use_allocate {
	() => {
		#[no_mangle]
		pub extern "C" fn handle_allocate(from: Address, cell: Address) {
			// This should not happen, since the wrapper method being used conforms to this practice
			PIPELINE_ALLOCATE.write().unwrap().replace(cell);
		}
	};
}

#[no_mangle]
pub fn allocate(to: Address, size: u32) -> Option<Address> {
	let msg_kind = CString::new("allocate").unwrap();

	send_message(to, msg_kind.as_ptr() as i32, ptr::addr_of!(size) as i32);
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

pub static PIPELINE_READ: RwLock<Option<u8>> = RwLock::new(None);

#[macro_export]
macro_rules! use_read {
	() => {
		#[no_mangle]
		pub extern "C" fn handle_read(from: Address, val: u8) {
			// This should not happen, since the wrapper method being used conforms to this practice
			PIPELINE_READ.write().unwrap().replace(val);
		}
	};
}

#[no_mangle]
pub extern "C" fn handle_read(from: Address, offset: u32) {
	assert_isowner!(from);

	let v = VAL
		.read()
		.unwrap()
		.get(offset as usize)
		.map(|byte| *byte)
		.unwrap();

	let msg_kind = std::ffi::CString::new("read").unwrap();
	send_message(from, msg_kind.as_ptr() as i32, ptr::addr_of!(v) as i32);
}

#[no_mangle]
pub fn read(to: Address, offset: u32) -> Option<u8> {
	let msg_kind = CString::new("read").unwrap();

	send_message(to, msg_kind.as_ptr() as i32, ptr::addr_of!(offset) as i32);
	PIPELINE_READ.write().unwrap().take()
}

#[no_mangle]
pub extern "C" fn handle_write(from: Address, offset: u32, val: u8) {
	assert_isowner!(from);

	if let Ok(mut lock) = VAL.write() {
		eassert!((offset as usize) < lock.len());

		lock[offset as usize] = val;
	}
}

#[no_mangle]
pub fn write(to: Address, offset: u32, val: u8) {
	let msg_kind = CString::new("write").unwrap();

	let offset: [u8; 4] = offset.to_le_bytes();
	let mut write_args: [u8; 5] = [0, 0, 0, 0, val];

	for (i, b) in offset.into_iter().enumerate() {
		write_args[i] = b;
	}

	send_message(
		to,
		msg_kind.as_ptr() as i32,
		ptr::addr_of!(write_args) as i32,
	);
}

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

#[no_mangle]
pub fn grow(to: Address, size: u32) {
	let msg_kind = CString::new("grow").unwrap();

	send_message(to, msg_kind.as_ptr() as i32, ptr::addr_of!(size) as i32);
}
