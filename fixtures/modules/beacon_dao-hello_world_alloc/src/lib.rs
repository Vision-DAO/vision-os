use beacon_dao_logger::{alias_service, info, use_alias_service, use_info};
use vision_derive::with_bindings;
use vision_utils::types::Address;
use wasm_bindgen::prelude::wasm_bindgen;

use_alias_service!();
use_info!();

#[wasm_bindgen]
pub fn init(parent: Address) {
	alias_service(3, "Test Actor".to_owned());
}

#[wasm_bindgen]
pub fn handle_test(from: Address) {
	info(3, "Bro".to_owned());
}
