use std::sync::RwLock;
use vision_derive::with_bindings;
use vision_utils::{
	actor::{address, spawn_actor},
	types::{Address, Callback},
};

/// The owner of the memory cell
static OWNER: RwLock<Option<Address>> = RwLock::new(None);

static VAL: RwLock<Vec<u8>> = RwLock::new(Vec::new());

macro_rules! eassert {
	($cond:expr, $callback:ident) => {
		if !$cond {
			$callback.call(1);

			return;
		}
	};
}

#[cfg(feature = "module")]
#[no_mangle]
pub extern "C" fn init(owner: Address) {
	if let Ok(mut lock) = OWNER.write() {
		lock.replace(owner);
	}
}

/// Creates a new memory cell.
#[no_mangle]
pub extern "C" fn alloc() -> Address {
	spawn_actor(address())
}

/// Appends the byte to the memory cell
#[no_mangle]
pub extern "C" fn append(val: u8) {
	if let Ok(mut lock) = VAL.write() {
		lock.push(val);
	}
}

#[no_mangle]
#[with_bindings]
pub extern "C" fn handle_write_chunk(
	from: Address,
	offset: u32,
	val: u128,
	size: u32,
	callback: Callback<u8>,
) {
	if let Ok(mut lock) = VAL.write() {
		eassert!(((offset + size) as usize) < lock.len(), callback);
		eassert!(size <= 16, callback);

		let bytes = val.to_le_bytes();

		// Write each byte
		for i in 0..size {
			lock[(offset + i) as usize] = bytes[i as usize];
		}
	}

	callback.call(0);
}

#[no_mangle]
#[with_bindings]
pub extern "C" fn handle_read_chunk(
	from: Address,
	offset: u32,
	size: u32,
	callback: Callback<u128>,
) {
	let offset = offset as usize;
	let size = size as usize;

	let bytes: [u8; 16] = VAL.read().unwrap().as_slice()[offset..(offset + size)]
		.try_into()
		.unwrap();
	callback.call(u128::from_le_bytes(bytes));
}

#[no_mangle]
#[with_bindings]
pub extern "C" fn handle_read(from: Address, offset: u32, callback: Callback<u8>) {
	callback.call(VAL.read().unwrap().get(offset as usize).copied().unwrap());
}

#[no_mangle]
#[with_bindings]
pub extern "C" fn handle_len(from: Address, callback: Callback<u32>) {
	callback.call(VAL.read().unwrap().len().try_into().unwrap());
}
