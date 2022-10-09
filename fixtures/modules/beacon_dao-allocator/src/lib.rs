use vision_utils::types::Address;
use wasm_bindgen::prelude::wasm_bindgen;

extern "C" {
	fn spawn_actor(addr: Address) -> Address;
	fn address() -> Address;
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

#[wasm_bindgen]
pub fn handle_allocate(from: Address, size: u32) {
	// Require that we are a manager to allocate memory
	eassert!(OWNER.is_none());
}
