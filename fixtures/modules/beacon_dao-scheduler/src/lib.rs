pub mod common;
pub mod runtime;

use runtime::gc::Rt;
use std::default::Default;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn start() {
	let mut rt = Rt::default();
}
