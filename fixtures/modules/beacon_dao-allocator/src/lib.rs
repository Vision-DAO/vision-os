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

#[wasm_bindgen]
pub fn handle_read(from: Address, offset: u32) {
	if let Some(val, msg_kind) = VAL.get(offset).zip(CString::new("read_ok").some()) {
		send_message(
			from,
			WasmPtr::from_native(msg_kind.as_ptr() as i32),
			WasmPtr::from_native((&val as *const u8) as i32),
		);
	} else {
		// Spawn and initialize a cell with enough space for an error message + null character
		let err_msg = "Failed to read from cell. Index out of bounds.";
		let err_msg_len = err_msg.len() + 1;

		let msg_kind = CString::new("allocate").unwrap();
		let err_buf = send_message(
			address(),
			WasmPtr::from_native(msg_kind.as_ptr() as i32),
			WasmPtr::from_native((&err_msg_len as *const u8) as i32),
		);

		// Copy the contents of the error message into the cell
		let msg_kind = CString::new("write").unwrap();

		for (i, b) in err_msg.as_bytes().enumerate() {
			// Write the character at position i to the buffer
			let args: [u8; 5] = [0, 0, 0, 0, b];

			for (i, b) in i.to_le_bytes().enumerate() {
				args[i] = b;
			}

			send_message(
				err_buf,
				WasmPtr::from_native((&msg_kind as *const u8) as i32),
				WasmPtr::from_native((&args as *const u8) as i32),
			);
		}

		let msg_kind = CString::new("read_err").unwrap();

		send_message(
			from,
			WasmPtr::from_native(msg_kind.as_ptr() as i32),
			err_buf,
		);
	}
}
