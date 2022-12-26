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

#[no_mangle]
pub extern "C" fn handle_bump(from: Address) {
	if from != address() {
		return;
	}

	let new_count = COUNT.fetch_add(2, Ordering::SeqCst) + 2;

	eval_js(
		DOM_ADDR,
		format!("document.getElementById('countLabel').innerText = {new_count};"),
		Callback::new(|_| {}),
	);
}
