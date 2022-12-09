use super::gc::Rt;

use crate::common::Address;
use wasmer::{FromToNativeWasmType, FunctionEnvMut, Memory32, WasmPtr};

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

#[cfg(feature = "wasm")]
#[wasm_bindgen]
extern "C" {
	#[wasm_bindgen(js_namespace = console)]
	pub fn log(s: &str);
}

#[cfg(feature = "cli")]
pub fn log(s: &str) {
	println!("{}", s);
}

impl Rt {
	// Gets the UTF-8 encoded C-string inside the child at ptr
	fn read_env_str(env: &FunctionEnvMut<(Address, Rt)>, ptr: i32) -> Option<String> {
		let children = env.data().1.children.read().ok()?;
		let logging_actor = children.get(env.data().0 as usize).map(Option::as_ref)??;
		let memory = logging_actor.instance.exports.get_memory("memory").ok()?;

		// Get the memory of the module calling log, and read the message they
		// want to log from memory
		let memory = memory.view(&env);
		<WasmPtr<u8, Memory32> as FromToNativeWasmType>::from_native(ptr)
			.read_utf8_string_with_nul(&memory)
			.ok()
	}

	/* Implementation of the web console log API */
	pub fn do_log_safe(env: FunctionEnvMut<(Address, Rt)>, msg: i32) -> Option<()> {
		let msg = Self::read_env_str(&env, msg)?;

		log(msg.as_str());
		Some(())
	}

	pub fn log_safe(env: FunctionEnvMut<(Address, Rt)>, msg: i32) {
		Self::do_log_safe(env, msg).unwrap();
	}

	/* Implementation of the DOM API */
	pub fn do_append_element_safe(
		env: FunctionEnvMut<(Address, Rt)>,
		kind: i32,
		src: i32,
	) -> Option<()> {
		// Get the source code of the HTML element to append
		let kind = Self::read_env_str(&env, kind)?;
		let src = Self::read_env_str(&env, src)?;

		// Get a handle on the document to append the element
		let window = web_sys::window()?;
		let document = window.document()?;
		let body = document.body()?;

		let node = document.create_element(kind.as_str()).ok()?;
		node.set_inner_html(src.as_str());

		body.append_child(&node).ok()?;

		Some(())
	}

	pub fn append_element_safe(env: FunctionEnvMut<(Address, Rt)>, kind: i32, src: i32) -> u8 {
		match Self::do_append_element_safe(env, kind, src) {
			Some(_) => 0,
			None => 1,
		}
	}
}
