use std::sync::RwLock;
use vision_derive::with_bindings;
use vision_utils::{
	actor::{address, spawn_actor},
	types::{Address, Callback},
};

/// The owner of the memory cell
static OWNER: RwLock<Option<Address>> = RwLock::new(None);

static VAL: RwLock<Vec<u8>> = RwLock::new(Vec::new());

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
pub extern "C" fn handle_read(from: Address, offset: u32, callback: Callback<u8>) {
	callback.call(VAL.read().unwrap().get(offset as usize).copied().unwrap());
}

#[no_mangle]
#[with_bindings]
pub extern "C" fn handle_len(from: Address, callback: Callback<u32>) {
	callback.call(VAL.read().unwrap().len().try_into().unwrap());
}
