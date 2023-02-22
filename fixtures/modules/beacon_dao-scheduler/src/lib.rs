pub mod common;
pub mod runtime;

use js_sys::Array;
use runtime::gc::Rt;
use std::{default::Default, panic, sync::Arc};
use vision_utils::types::DISPLAY_MANAGER_ADDR;
use wasm_bindgen::prelude::wasm_bindgen;

// Global instance of the runtime that external modules can use to interact
// with.
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
		true,
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

	// HTTP client module
	RT.spawn(
		None,
		include_bytes!("../../target/wasm32-unknown-unknown/release/beacon_dao_fetch.wasm"),
		true,
	)
	.expect("Failed to start HTTP client.");

	// Web3 client module
	RT.spawn(
		None,
		include_bytes!("../../target/wasm32-unknown-unknown/release/beacon_dao_web3.wasm"),
		true,
	)
	.expect("Failed to start web3 client.");

	// Permission delegate
	RT.spawn(
		None,
		include_bytes!(
			"../../target/wasm32-unknown-unknown/release/beacon_dao_permissions_consent.wasm"
		),
		true,
	)
	.expect("Failed to start permissions delegate");

	RT.impulse(None, DISPLAY_MANAGER_ADDR, "display_login", &[][..])
		.expect("Failed to login");
}

/// Sends a message to the global runtime instance, pretending that the message was sent from the from address provided.
#[wasm_bindgen]
pub fn impulse(from: u32, to: u32, msg_name: String, params: Array) {
	#[wasm_bindgen]
	extern "C" {
		#[wasm_bindgen(js_namespace = console)]
		pub fn log(s: &str);
	}

	RT.impulse_js(Some(from), Some(to), msg_name.as_str(), params)
		.unwrap();
}

/// Drives the runtime to completion.
#[wasm_bindgen]
pub fn poll() {
	if let Err(e) = RT.poll() {
		panic!("event loop panicked with: {}", e);
	}
}
