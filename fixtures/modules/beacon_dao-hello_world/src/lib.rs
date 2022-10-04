use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;
use web_sys::console::log_1;

#[cfg(test)]
pub mod tests;

#[wasm_bindgen]
pub fn start() {
	log_1(&JsValue::from_str("Hello, world!"));
}
