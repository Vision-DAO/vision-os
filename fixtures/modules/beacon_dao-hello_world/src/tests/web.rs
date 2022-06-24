use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

// Run tests inside a browser instead of nodejs
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_main() {
    super::super::start();
}
