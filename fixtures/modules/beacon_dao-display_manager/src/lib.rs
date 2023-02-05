use beacon_dao_dom::{create_element, eval_js};
use std::sync::{
	atomic::{AtomicUsize, Ordering},
	Arc,
};
use vision_derive::with_bindings;
use vision_utils::{
	actor::address,
	types::{Address, Callback, DOM_ADDR},
};

lazy_static::lazy_static! {
	static ref COUNT: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
}

#[no_mangle]
pub extern "C" fn handle_display_login(from: Address) {
	create_element(
		DOM_ADDR,
		String::from("div"),
		include_str!("./index.html").to_owned(),
		Callback::new(|_| {
			eval_js(
				DOM_ADDR,
				include_str!("./index.js").to_owned(),
				Callback::new(|_| {}),
			);
		}),
	);
}

/// Loads the config profile at the specified Ethereum address.
pub extern "C" fn handle_login_as(from: Address, username: String, callback: Callback<usize>) {}
