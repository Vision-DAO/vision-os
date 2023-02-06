use beacon_dao_dom::eval_js;
use beacon_dao_permissions::{beacon_dao_allocator::read, has_permission, register_permission};
use js_sys::JSON;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen;
use std::{collections::HashMap, sync::RwLock};
use vision_derive::with_bindings;
use vision_utils::types::{
	Address, Callback, DISPLAY_MANAGER_ADDR, DOM_ADDR, EXIT_FAILURE, EXIT_SUCCESS,
	MOCK_ALLOCATOR_ADDR, PERM_ADDR,
};

const PERM: &'static str = "make_http_request";
const DESCRIPTION: &'static str = "Allows the app to make a request to the web.";

/// Callbacks that should be run after the fetch() promise is returned.
pub static TASKS: RwLock<Vec<Option<Callback<Result<Response, ()>>>>> = RwLock::new(Vec::new());

/// HTTP methods available.
#[derive(Serialize, Deserialize)]
pub enum Method {
	POST,
	GET,
	PATCH,
}

/// Settings which may be provided, but have defaults, for an HTTP request.
pub struct OptionsBuilder<T: Serialize> {
	pub method: Option<Method>,
	pub headers: Option<HashMap<String, String>>,
	pub body: Option<T>,
}

impl<T: Serialize> From<OptionsBuilder<T>> for Options {
	fn from(mut opts: OptionsBuilder<T>) -> Self {
		Options {
			method: opts.method.take().unwrap_or(Method::GET),
			headers: opts.headers.take().unwrap_or(HashMap::new()),
			body: serde_json::to_string(&opts.body).ok(),
		}
	}
}

/// Settings for an HTTP request.
#[derive(Serialize, Deserialize)]
pub struct Options {
	pub method: Method,
	pub headers: HashMap<String, String>,
	pub body: Option<String>,
}

/// An HTTP response.
#[derive(Serialize, Deserialize)]
pub struct Response {
	pub body: Option<Vec<u8>>,
	pub status: usize,
}

#[cfg(feature = "module")]
#[no_mangle]
pub extern "C" fn init(owner: Address) {
	register_permission(
		PERM_ADDR,
		PERM.to_owned(),
		DESCRIPTION.to_owned(),
		Callback::new(|_| {}),
	);
}

/// Calls the fetch window method with the provided arguments.
#[no_mangle]
#[with_bindings]
pub extern "C" fn handle_fetch(
	from: Address,
	resource: String,
	opts: Options,
	callback: Callback<Result<Response, ()>>,
) {
	// Check that the user can make HTTP requests
	has_permission(
		PERM_ADDR,
		from,
		PERM.into(),
		Callback::new(move |has_perm: bool| {
			// Only let permissioned users make HTTP requests
			if !has_perm && from != DISPLAY_MANAGER_ADDR {
				callback.call(Err(()));

				return;
			}

			// Save the callback to be run after the fetch() call is done
			let slot = {
				let mut slots = if let Ok(lock) = TASKS.write() {
					lock
				} else {
					callback.call(Err(()));

					return;
				};

				let slot = slots.len();
				slots.push(Some(callback));

				slot
			};

			let opts_ser = if let Ok(ser) = serde_json::to_string(&opts) {
				ser
			} else {
				let mut slots = if let Ok(slots) = TASKS.write() {
					slots
				} else {
					return;
				};

				let cb = if let Some(cb) = slots.get_mut(slot).and_then(|task| task.take()) {
					cb
				} else {
					return;
				};

				cb.call(Err(()));

				return;
			};

			eval_js(
				DOM_ADDR,
				format!(
					"fetch({}, {}).then((resp) => impulse(address(), 'fetch_resp', [{}, resp]))",
					resource, opts_ser, slot,
				),
				Callback::new(move |stat| {
					if stat == EXIT_FAILURE {
						let mut slots = if let Ok(slots) = TASKS.write() {
							slots
						} else {
							return;
						};

						let cb = if let Some(cb) = slots.get_mut(slot).and_then(|task| task.take())
						{
							cb
						} else {
							return;
						};

						cb.call(Err(()));
					}

					// The task will complete in a call to fetch_resp
				}),
			);
		}),
	);
}

/// Calls the registered task callback for the fetch call.
#[no_mangle]
#[with_bindings]
pub extern "C" fn handle_fetch_resp(
	from: Address,
	cb_id: u32,
	json_cell: Address,
	callback: Callback<u32>,
) {
	let task = if let Some(task) = TASKS
		.write()
		.ok()
		.and_then(|mut tasks| tasks.get_mut(cb_id as usize).and_then(|task| task.take()))
	{
		task
	} else {
		callback.call(EXIT_FAILURE);

		return;
	};

	// Wait until all {} have been closed
	let buff = String::new();
	let unmatched = Vec::new();

	fn pop_c(
		mut buf: String,
		mut unmatched: Vec<()>,
		pos: u32,
		cell: Address,
		task: Callback<Result<Response, ()>>,
	) {
		read(
			cell,
			pos,
			Callback::new(move |c: u8| {
				let c = c as char;

				if c == '{' {
					unmatched.push(());
				} else if c == '}' {
					unmatched.pop();
				}

				buf.push(c);

				// The JS object finished transmitting
				if unmatched.len() == 0 {
					task.call(serde_json::from_str(buf.as_str()).map_err(|_| ()));

					return;
				}

				// Still more to go
				pop_c(buf, unmatched, pos + 1, cell, task);
			}),
		);
	}

	pop_c(buff, unmatched, 0, json_cell, task);
}
