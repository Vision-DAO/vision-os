use wasm_bindgen::prelude::wasm_bindgen;

pub mod tests;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn start() {
    log("Hello, world!");
}
