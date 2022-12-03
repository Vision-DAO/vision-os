use std::panic;

use beacon_dao_scheduler::runtime::{gc::Rt, Runtime};

pub fn start() {
	panic::set_hook(Box::new(console_error_panic_hook::hook));

	let rt = Rt::default();

	// Permissions service
	rt.spawn(
		None,
		include_bytes!("../../target/wasm32-unknown-unknown/release/beacon_dao_allocator.wasm"),
		true,
	)
	.expect("Failed to start permissions service");

	// Allocator API
	rt.spawn(
		None,
		include_bytes!("../../target/wasm32-unknown-unknown/release/beacon_dao_allocator.wasm"),
		true,
	)
	.expect("Failed to start allocator service");

	// Logger API
	rt.spawn(
		None,
		include_bytes!("../../target/wasm32-unknown-unknown/release/beacon_dao_allocator.wasm"),
		true,
	)
	.expect("Failed to start logging service");

	// Logging service
	rt.spawn(
		None,
		include_bytes!("../../target/wasm32-unknown-unknown/release/beacon_dao_allocator.wasm"),
		true,
	)
	.expect("Failed to start logging service");

	// Default allocator service
	rt.spawn(
		None,
		include_bytes!("../../target/wasm32-unknown-unknown/release/beacon_dao_allocator.wasm"),
		true,
	)
	.expect("Failed to start allocator service");

	// Hello world service
	// TODO: Remove
	rt.spawn(
		None,
		include_bytes!("../../target/wasm32-unknown-unknown/release/beacon_dao_test_ping.wasm"),
		true,
	)
	.expect("Failed to start test service");
	rt.spawn(
		None,
		include_bytes!("../../target/wasm32-unknown-unknown/release/beacon_dao_test_pong.wasm"),
		true,
	)
	.expect("Failed to start test service");

	// Test out the hello world module
	rt.impulse("ping", vec![]).unwrap();
}
