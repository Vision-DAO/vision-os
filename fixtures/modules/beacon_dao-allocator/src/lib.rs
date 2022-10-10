use vision_utils::types::Address;
use wasm_bindgen::prelude::wasm_bindgen;

extern "C" {
	fn spawn_actor(addr: Address) -> Address;
	fn address() -> Address;
	fn send_message(addr: Address, msg_name_buf: &str, msg_buf: WasmPtr<u8, Array>);
}

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

struct InitMessage {}

#[wasm_bindgen]
pub fn handle_allocate(from: Address, size: u32) {
	// Require that we are a manager to allocate memory
	eassert!(OWNER.is_none());

	let memcell = spawn_actor(address());

	send_message(memcell, "")
}

#[wasm_bindgen]
pub fn init(owner: Address) {
	OWNER.replace(Some(owner));
}

#[wasm_bindgen]
pub fn handle_read(from: Address, offset: u32) {
	if let Some(val) = VAL.get(offset) {
		send_message(from, "read_ok", val);
	} else {
		let err = spawn_actor(address());
		send_message(from, "read_err", 
	}
}

/// Initializes the memory cell from a string.
macro_rules! from_str {
	($s:expr) => {
		let cell = send_message(
	}
}
