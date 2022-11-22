use super::gc::Rt;

use crate::common::Address;
use wasm_bindgen::prelude::wasm_bindgen;
use wasmer::{FromToNativeWasmType, FunctionEnvMut, Memory32, WasmPtr};

#[wasm_bindgen]
extern "C" {
	#[wasm_bindgen(js_namespace = console)]
	fn log(s: &str);
}

impl Rt {
	/* Implementation of the web console log API */
	pub fn do_log_safe(env: FunctionEnvMut<(Address, Rt)>, msg: i32) -> Option<()> {
		let children = env.data().1.children.read().ok()?;
		let logging_actor = children.get(env.data().0 as usize).map(Option::as_ref)??;
		let memory = logging_actor.instance.exports.get_memory("memory").ok()?;

		// Get the memory of the module calling log, and read the message they
		// want to log from memory
		let memory = memory.view(&env);
		let msg = <WasmPtr<u8, Memory32> as FromToNativeWasmType>::from_native(msg)
			.read_utf8_string_with_nul(&memory)
			.ok()?;

		log(msg.as_str());
		Some(())
	}

	pub fn log_safe(env: FunctionEnvMut<(Address, Rt)>, msg: i32) {
		Self::do_log_safe(env, msg).unwrap();
	}
}
