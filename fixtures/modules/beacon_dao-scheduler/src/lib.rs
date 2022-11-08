pub mod common;
pub mod runtime;

use runtime::{gc::Rt, Runtime};
use std::{default::Default, panic};
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn start() {
	panic::set_hook(Box::new(console_error_panic_hook::hook));

	let rt = Rt::default();

	// Permissions service
	rt.spawn(
		None,
		include_bytes!("../../target/wasm32-unknown-unknown/release/beacon_dao_permissions.wasm"),
		true,
	)
	.expect("Failed to start permissions service");

	// Allocator service
	rt.spawn(
		None,
		include_bytes!("../../target/wasm32-unknown-unknown/release/beacon_dao_allocator.wasm"),
		true,
	)
	.expect("Failed to start allocator service");

	// Logging service
	rt.spawn(
		None,
		include_bytes!("../../target/wasm32-unknown-unknown/release/beacon_dao_logger.wasm"),
		true,
	)
	.expect("Failed to start logging service");

	// Hello world service
	// TODO: Remove
	rt.spawn(
		None,
		include_bytes!(
			"../../target/wasm32-unknown-unknown/release/beacon_dao_hello_world_alloc.wasm"
		),
		true,
	)
	.expect("Failed to start hello world service");

	// Test out the hello world module
	rt.impulse("test", vec![]);
}
