pub mod common;
pub mod runtime;

use runtime::{gc::Rt, Runtime};
use std::{default::Default, panic, sync::Arc};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};

/// Global instance of the runtime that external modules can use to interact
/// with.
lazy_static::lazy_static! {
	static ref RT: Arc<Rt> = Arc::new(Rt::default());
}

#[wasm_bindgen]
pub fn start() {
	panic::set_hook(Box::new(console_error_panic_hook::hook));

	// Permissions service
	RT.spawn(
		None,
		include_bytes!("../../target/wasm32-unknown-unknown/release/beacon_dao_permissions.wasm"),
		false,
	)
	.expect("Failed to start permissions service");

	// Allocator API
	RT.spawn(
		None,
		include_bytes!(
			"../../target/wasm32-unknown-unknown/release/beacon_dao_allocator_manager.wasm"
		),
		false,
	)
	.expect("Failed to start allocator service");

	// Logger API
	RT.spawn(
		None,
		include_bytes!(
			"../../target/wasm32-unknown-unknown/release/beacon_dao_logger_manager.wasm"
		),
		false,
	)
	.expect("Failed to start logging service");

	// Logging service
	RT.spawn(
		None,
		include_bytes!("../../target/wasm32-unknown-unknown/release/beacon_dao_logger.wasm"),
		true,
	)
	.expect("Failed to start logging service");

	// Default allocator service
	RT.spawn(
		None,
		include_bytes!("../../target/wasm32-unknown-unknown/release/beacon_dao_allocator.wasm"),
		false,
	)
	.expect("Failed to start allocator service");

	// Default DOM service
	RT.spawn(
		None,
		include_bytes!("../../target/wasm32-unknown-unknown/release/beacon_dao_dom.wasm"),
		true,
	)
	.expect("Failed to start DOM service.");

	// Display manager service
	RT.spawn(
		None,
		include_bytes!(
			"../../target/wasm32-unknown-unknown/release/beacon_dao_display_manager.wasm"
		),
		true,
	)
	.expect("Failed to start display manager.");

	// Mock allocator module
	RT.spawn(
		None,
		include_bytes!("../../target/wasm32-unknown-unknown/release/beacon_dao_mock_alloc.wasm"),
		true,
	)
	.expect("Failed to start mock allocator.");
}

/// Sends a message to the global runtime instance, pretending that the message was sent from the from address provided.
#[wasm_bindgen]
pub fn impulse(from: u32, msg_name: &str, params: Vec<JsValue>) {
	RT.impulse_js(Some(from), msg_name, params).unwrap();
}
