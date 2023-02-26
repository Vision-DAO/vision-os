use beacon_dao_dom::eval_js;
use beacon_dao_permissions::{
	beacon_dao_allocator::{len, read},
	has_permission, register_permission,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{
	collections::HashMap,
	sync::{
		atomic::{AtomicUsize, Ordering},
		Arc, Mutex, RwLock,
	},
};
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
			body: opts.body.and_then(|body| serde_json::to_string(&body).ok()),
		}
	}
}

/// Settings for an HTTP request.
#[derive(Serialize, Deserialize)]
pub struct Options {
	pub method: Method,
	pub headers: HashMap<String, String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub body: Option<String>,
}

/// An HTTP response.
#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
	pub body: Option<Map<String, Value>>,
	pub status: usize,
}

#[cfg(feature = "module")]
#[no_mangle]
pub extern "C" fn init(owner: Address) {
	extern "C" {
		fn print(s: i32);
	}

	let msg = std::ffi::CString::new(format!("registsering {} {}", PERM, DESCRIPTION)).unwrap();

	unsafe {
		print(msg.as_ptr() as i32);
	}
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
					"fetch('{}', {})
						  .then((resp) => impulse(address(), 'fetch_resp', {}, {{ status: resp.status, body: resp.body }}))
						  .catch((err) => impulse(address(), 'fetch_resp_err', {}, err))",
					resource, opts_ser, slot, slot,
				),
				Callback::new(move |stat| {
					if stat as u32 == EXIT_FAILURE {
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
	// Wait until all characters have been read
	len(
		json_cell,
		Callback::new(move |to_read| {
			// Make a string of n spaces
			let buff = Arc::new(Mutex::new(
				(0..to_read).map(|_| '\0').collect::<Vec<char>>(),
			));
			let n_read = Arc::new(AtomicUsize::new(0));

			// Concurrently read n bytes
			for i in 0..to_read {
				let buff = buff.clone();
				let n_read = n_read.clone();

				read(
					json_cell,
					i,
					Callback::new(move |c: u8| {
						*buff.lock().unwrap().get_mut(i as usize).unwrap() = c as char;

						// We read the last character
						if n_read.fetch_add(1usize, Ordering::SeqCst) == (to_read as usize) - 1usize
						{
							let task = if let Some(task) =
								TASKS.write().ok().and_then(|mut tasks| {
									tasks.get_mut(cb_id as usize).and_then(|task| task.take())
								}) {
								task
							} else {
								return;
							};

							let read_str: String = buff.lock().unwrap().drain(..).collect();
							let read_resp =
								if let Ok(resp) = serde_json::from_str(read_str.as_str()) {
									resp
								} else {
									task.call(Err(()));

									return;
								};

							task.call(Ok(read_resp));
						}
					}),
				);
			}
		}),
	);
}

/// Calls the registered task callback for the erronous fetch call.
#[no_mangle]
#[with_bindings]
pub extern "C" fn handle_fetch_resp_err(
	from: Address,
	cb_id: u32,
	json_cell: Address,
	callback: Callback<u32>,
) {
	// Read message here
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

	task.call(Err(()));
}
