use beacon_dao_dom::{create_element, eval_js};
use beacon_dao_fetch::{fetch, OptionsBuilder, Response};
use serde::{Deserialize, Serialize};
use std::sync::{
	atomic::{AtomicUsize, Ordering},
	Arc, RwLock,
};
use vision_derive::with_bindings;
use vision_utils::{
	actor::address,
	types::{Address, Callback, DOM_ADDR, FETCH_ADDR, PERM_ADDR},
};

/// Kinds of dialogues supported by the display manager.
#[derive(Serialize, Deserialize)]
pub enum DialogueKind {
	/// A dialogue with no options
	Alert,

	/// A dialogue with only one option
	Affirm(String),

	/// A dialogue with two options
	Choice(String, String),
}

/// Loads the config profile at the specified Ethereum address.
#[no_mangle]
#[with_bindings]
pub extern "C" fn handle_login_as(from: Address, username: String, callback: Callback<u32>) {
	fetch(
		FETCH_ADDR,
		String::from("https://google.com"),
		OptionsBuilder::<String> {
			method: None,
			headers: None,
			body: None,
		}
		.into(),
		Callback::new(|_| {}),
	);
}

/// System dialogue callbacks.
pub static TASKS: RwLock<Vec<Option<Callback<u32>>>> = RwLock::new(Vec::new());

/// Displays a dialogue on the screen with the given title, description, and nature.
/// Calls the callback when the dialogue is dismissed, with a return value
/// of 0 representing standard dismissal, and 1 representing the user choosing the
/// right binary choice, and 2 representing an error.
#[no_mangle]
#[with_bindings]
pub extern "C" fn handle_system_dialogue(
	from: Address,
	title: String,
	description: String,
	kind: DialogueKind,
	callback: Callback<u32>,
) {
	if from != PERM_ADDR && from != address() {
		callback.call(2);

		return;
	}

	// Register the user's callback for later
	let slot = {
		let mut slots = if let Ok(lock) = TASKS.write() {
			lock
		} else {
			callback.call(2);

			return;
		};

		let slot = slots.len();
		slots.push(Some(callback));

		slot
	};

	match kind {
		DialogueKind::Choice(yes, no) => create_element(
			DOM_ADDR,
			String::from("div"),
			include_str!("./dialogue.html")
				.to_owned()
				.replace("#yes#", &yes)
				.replace("#no#", &no)
				.replace("#title#", &title)
				.replace("#desc#", &description),
			Callback::new(move |_| {
				eval_js(
					DOM_ADDR,
					include_str!("./dialogue.js")
						.to_owned()
						.replace("#cbid#", &slot.to_string()),
					Callback::new(|_| {}),
				);
			}),
		),
		// TODO:
		_ => unimplemented!(),
	}
}

/// Handle the user clicking on yes or no, and return that status code to the end user.
#[no_mangle]
#[with_bindings]
pub extern "C" fn handle_system_dialogue_resp(
	from: Address,
	value: u32,
	cb_id: u32,
	cb: Callback<u32>,
) {
	let task = if let Some(task) = TASKS
		.write()
		.ok()
		.and_then(|mut tasks| tasks.get_mut(cb_id as usize).and_then(|task| task.take()))
	{
		task
	} else {
		cb.call(1);

		return;
	};

	task.call(value);
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
