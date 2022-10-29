pub mod common;
pub mod runtime;

use runtime::{gc::Rt, Runtime};
use std::default::Default;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn start() {
	let mut rt = Rt::default();

	// Permissions service
	rt.spawn(
		None,
		include_bytes!("../../target/wasm32-unknown-unknown/release/beacon_dao_permissions.wasm"),
	)
	.unwrap("Failed to start permissions service");

	// Allocator service
	rt.spawn(
		None,
		include_bytes!("../../target/wasm32-unknown-unknkown/release/beacon_dao_allocator.wasm"),
	)
	.unwrap("Failed to start allocator service");

	// Logging service
	rt.spawn(
		None,
		include_bytes!("../../target/wasm32-unknown-unknown/release/beacon_dao_logger.wasm"),
	)
	.expect("Failed to start logging service");

	// Hello world service
	// TODO: Remove
	rt.spawn(
		None,
		include_bytes!("../../target/wasm32-unknown-unknown/release/hello_world_alloc.wasm"),
	);

	// Test out the hello world module
	rt.impulse("test", vec![]);
}
