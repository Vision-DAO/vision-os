use beacon_dao_dom::create_element;
use vision_derive::with_bindings;
use vision_utils::types::{Address, Callback, DOM_ADDR};

#[no_mangle]
pub extern "C" fn handle_display_login(from: Address) {
	create_element(
		DOM_ADDR,
		String::from("div"),
		include_str!("./index.html").to_owned(),
		Callback::new(|_| {}),
	);
}
