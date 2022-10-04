/// Writes the given message to the console, with the name of the source actor.
#[wasm_bindgen]
pub fn info(msg: impl AsRef<str> + Display) {}
