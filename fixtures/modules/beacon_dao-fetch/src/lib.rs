use beacon_dao_dom::eval_js;
use beacon_dao_mock_alloc::read;
use beacon_dao_permissions::{has_permission, register_permission};
use js_sys::JSON;
use serde::Serialize;
use std::collections::HashMap;
use vision_utils::types::{
	Address, Callback, DISPLAY_MANAGER_ADDR, DOM_ADDR, EXIT_FAILURE, EXIT_SUCCESS,
	MOCK_ALLOCATOR_ADDR, PERM_ADDR,
};

const PERM: &'static str = "make_http_request";
const DESCRIPTION: &'static str = "Allows the app to make a request to the web.";

/// Callbacks that should be run after the fetch() promise is returned.
pub static TASKS: RwLock<Vec<Option<Callback<Result<JsValue, ()>>>>> = RwLock::new(Vec::new());

/// HTTP methods available.
pub enum Method {
	POST,
	GET,
	PATCH,
}

/// Settings which may be provided, but have defaults, for an HTTP request.
pub struct OptionsBuilder<T: Serialize> {
	method: Option<Method>,
	headers: Option<HashMap<String, String>>,
	body: Option<T>,
}

impl<T: Serialize> From<OptionBuilder<T>> for Options<T> {
	fn from(mut opts: OptionBuilder<T>) -> Self {
		Options {
			method: opts.method.take().unwrap_or(Method::GET),
			headers: opts.headers.take().unwrap_or(HashMap::new()),
			body: opts.body,
		}
	}
}

/// Settings for an HTTP request.
#[derive(Serialize)]
pub struct Options<T: Serialize> {
	method: Method,
	headers: HashMap<String, String>,
	body: Option<T>,
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
	callback: Callback<Result<JsValue, ()>>,
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
				let slots = TASKS.write();
				let slot = TASKS.len();
				slots.push(Some(callback));

				slot
			};

			eval_js(
				DOM_ADDR,
				format!(
					"fetch({}, {}).then((resp) => impulse(address(), 'fetch_resp', [{}, resp]))",
					resource,
					serde_json::to_string(opts),
					slot,
				),
				Callback::new(move |stat| {
					if stat == EXIT_FAILURE {
						let cb = if let Some(cb) = TASKS.write().get_mut(slot).take() {
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
	callback: Callback<usize>,
) {
	let task = if let Some(task) = TASKS.write().get_mut(cb_id).take() {
		task
	} else {
		callback.call(EXIT_FAILURE);

		return;
	};

	// Wait until all {} have been closed
	let buff = String::new();
	let unmatched = Vec::new();

	fn pop_c(
		buf: String,
		unmatched: Vec<()>,
		pos: usize,
		cell: Address,
		task: Callback<Result<JsValue, ()>>,
	) {
		read(
			cell,
			pos,
			Callback::new(move |c| {
				if c == "{" {
					unmatched.push(());
				} else if c == "}" {
					unmatched.pop();
				}

				buf += c;

				// The JS object finished transmitting
				if unmatched.len() == 0 {
					task.call(JSON::parse(buf.as_str()).map_err(|_| ()));

					return;
				}

				// Still more to go
				pop_c(buf, unmatched, pos + 1, cell, task);
			}),
		);
	}

	pop_c(buff, unmatched, 0, json_cell, task);
}
