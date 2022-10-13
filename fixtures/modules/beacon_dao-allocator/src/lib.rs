use std::ffi::CString;
use vision_utils::{
	actor::{address, send_message, spawn_actor},
	types::Address,
	with_result_message,
};
use wasm_bindgen::prelude::wasm_bindgen;
use wasmer::{Array, FromToNativeWasmType, WasmPtr};

macro_rules! eassert {
	($cond:expr) => {
		if !$cond {
			return;
		}
	};
}

/// An error encountered while processing an allocator message.
#[derive(Serialize, Deserialize)]
pub enum Error {
	OutOfBounds,
}

/// The owner of the memory cell. If this is the manager allocating cells, no
/// owner is specified.
static mut OWNER: Option<Address> = None;

/// The contents of the memory cell.
static mut VAL: Vec<u8> = Vec::new();

#[wasm_bindgen]
pub fn handle_allocate(from: Address, size: u32) {
	// Require that we are a manager to allocate memory
	eassert!(OWNER.is_none());

	let memcell = spawn_actor(address());

	// Grow the memory cell by the specified size
	let msg_kind = CString::new("grow").unwrap();
	send_message(
		memcell,
		WasmPtr::from_native(msg_kind.as_ptr() as i32),
		WasmPtr::from_native((&size as *const u8) as i32),
	);
}

#[wasm_bindgen]
pub fn init(owner: Address) {
	OWNER.replace(Some(owner));
}

#[with_result_message]
#[wasm_bindgen]
pub fn handle_read(from: Address, offset: u32) -> Result<u8, Error> {
	VAL.get(offset).ok_or(Error::OutOfBounds)
}
