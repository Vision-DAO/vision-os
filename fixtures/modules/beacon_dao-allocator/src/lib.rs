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
	callback.call(do_read(offset));
}

pub fn do_read(offset: u32) -> u8 {
	VAL.read()
		.unwrap()
		.get(offset as usize)
		.map(|byte| *byte)
		.unwrap()
}

#[no_mangle]
#[with_bindings(self)]
pub extern "C" fn handle_read_chunk(
	from: Address,
	offset: u32,
	size: u32,
	callback: Callback<u128>,
) {
	callback.call(do_read_chunk(offset, size));
}

fn do_read_chunk(offset: u32, size: u32) -> u128 {
	let offset = offset as usize;
	let size = size as usize;

	let bytes: [u8; 16] = VAL.read().unwrap().as_slice()[offset..(offset + size)]
		.try_into()
		.unwrap();

	u128::from_le_bytes(bytes)
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

	do_write(offset, val);

	callback.call(0);
}

pub fn do_write(offset: u32, val: u8) {
	if let Ok(mut lock) = VAL.write() {
		lock[offset as usize] = val;
	}
}

#[no_mangle]
#[with_bindings(self)]
pub extern "C" fn handle_write_chunk(
	from: Address,
	offset: u32,
	val: u128,
	size: u32,
	callback: Callback<u8>,
) {
	assert_isowner!(from, callback);

	callback.call(do_write_chunk(offset, val, size));
}

fn do_write_chunk(offset: u32, val: u128, size: u32) -> u8 {
	if let Ok(mut lock) = VAL.write() {
		if size > 16 || ((offset + size) as usize) < lock.len() {
			return 1;
		}

		let bytes = val.to_le_bytes();

		// Write each byte
		for i in 0..size {
			lock[(offset + i) as usize] = bytes[i as usize];
		}
	}

	0
}

#[no_mangle]
#[with_bindings(self)]
pub extern "C" fn handle_grow(from: Address, size: u32, callback: Callback<u8>) {
	assert_isowner!(from, callback);

	do_grow(size);

	callback.call(0);
}

fn do_grow(size: u32) {
	if let Ok(mut lock) = VAL.write() {
		// Add `size` zero bytes to the buffer
		for _ in 0..size {
			lock.push(0);
		}
	}
}

#[no_mangle]
#[with_bindings(self)]
pub extern "C" fn handle_len(from: Address, callback: Callback<u32>) {
	callback.call(VAL.read().unwrap().len().try_into().unwrap());
}

#[cfg(test)]
mod tests {
	use super::*;
	use serial_test::serial;

	fn clear() {
		VAL.write().unwrap().clear();
	}

	#[test]
	#[serial]
	fn test_write() {
		clear();

		do_grow(1);
		do_write(0, 69);
		assert_eq!(do_read(0), 69);

		do_grow(2);
		do_write(1, 4);
		do_write(2, 20);
		assert_eq!(do_read(1), 4);
		assert_eq!(do_read(2), 20);
		assert_eq!(do_read(0), 69);
	}

	#[test]
	#[serial]
	fn test_chunks() {
		clear();

		// Try writing some chunks
		let bytes: [u8; 16] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
		let packed = u128::from_le_bytes(bytes);

		do_grow(16);
		assert_eq!(do_write_chunk(0, packed, 16), 0);

		for i in 0..16u8 {
			assert_eq!(do_read(i as u32), i + 1);
		}

		// Try reading the chunks
		let bytes2: [u8; 16] = do_read_chunk(0, 16).to_le_bytes();

		assert_eq!(bytes, bytes2);
	}
}
