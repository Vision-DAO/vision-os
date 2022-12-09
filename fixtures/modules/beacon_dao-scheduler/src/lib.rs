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
		false,
	)
	.expect("Failed to start permissions service");

	// Allocator API
	rt.spawn(
		None,
		include_bytes!(
			"../../target/wasm32-unknown-unknown/release/beacon_dao_allocator_manager.wasm"
		),
		false,
	)
	.expect("Failed to start allocator service");

	// Logger API
	rt.spawn(
		None,
		include_bytes!(
			"../../target/wasm32-unknown-unknown/release/beacon_dao_logger_manager.wasm"
		),
		false,
	)
	.expect("Failed to start logging service");

	// Logging service
	rt.spawn(
		None,
		include_bytes!("../../target/wasm32-unknown-unknown/release/beacon_dao_logger.wasm"),
		true,
	)
	.expect("Failed to start logging service");

	// Default allocator service
	rt.spawn(
		None,
		include_bytes!("../../target/wasm32-unknown-unknown/release/beacon_dao_allocator.wasm"),
		false,
	)
	.expect("Failed to start allocator service");

	// Default DOM service
	rt.spawn(
		None,
		include_bytes!("../../target/wasm32-unknown-unknown/release/beacon_dao_dom.wasm"),
		true,
	)
	.expect("Failed to start DOM service.");

	// Display manager service
	rt.spawn(
		None,
		include_bytes!(
			"../../target/wasm32-unknown-unknown/release/beacon_dao_display_manager.wasm"
		),
		true,
	)
	.expect("Failed to start display manager.");

	// Test out the hello world module
	rt.impulse("display_login", vec![]).unwrap();
}
