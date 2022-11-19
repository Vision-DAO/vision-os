use super::gc::Rt;

use web_sys::CanvasRenderingContext2d;

#[wasm_bindgen]
extern "C" {
	#[wasm_bindgen(js_namespace = console)]
	fn log(s: &str);
}

impl Rt {
	/* Implementation of the web console log API */
	fn do_log_safe(env: FunctionEnvMut<(Address, Rt)>, msg: i32) -> Option<()> {
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

	fn log_safe(env: FunctionEnvMut<(Address, Rt)>, msg: i32) {
		Self::do_log_safe(env, msg).unwrap();
	}

	/* Implementation of the CanvasRenderingContext2D.fillRect API */
	fn fillrect_safe(
		env: FunctionEnvMut<(CanvasRenderingContext2d, Address, Rt)>,
		x: f64,
		y: f64,
		width: f64,
		height: f64,
	) -> Option<()> {
		env.data().0.fill_rect(x, y, width, height);

		Some(())
	}
}
