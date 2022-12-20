use std::panic;

use beacon_dao_scheduler::runtime::gc::Rt;

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

	// Hello world service
	rt.spawn(
		None,
		include_bytes!(
			"../../target/wasm32-unknown-unknown/release/beacon_dao_hello_world_alloc.wasm"
		),
		true,
	)
	.expect("Failed to start hello world service");

	// Test out the hello world module
	rt.impulse_all(None, "ping", vec![]);
}
