pub mod common;
pub mod runtime;

use runtime::{gc::Rt, Runtime};
use std::default::Default;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn start() {
	let mut rt = Rt::default();

	// Logging service
	rt.spawn(
		None,
		include_bytes!("../../target/wasm32-unknown-unknown/release/beacon_dao_logger.wasm"),
	)
	.unwrap();
}
