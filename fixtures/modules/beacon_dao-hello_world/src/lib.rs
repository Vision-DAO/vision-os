use wasm_bindgen::prelude::wasm_bindgen;
use web_sys::console::log_1;
use wasm_bindgen::JsValue;

#[cfg(test)]
pub mod tests;

#[wasm_bindgen]
pub fn start() {
    log_1(&JsValue::from_str("Hello, world!"));
}
