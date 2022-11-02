use wasmer::{Memory, ValueType, WasmPtr};

/// Comes from wasmer. Not exported in JS target.
pub fn get_utf8_string_with_nul<'a, T: Copy + ValueType>(
	ptr: WasmPtr<T>,
	memory: &'a Memory,
) -> Option<String> {
	unsafe {
		memory.view()[(ptr.offset() as usize)..]
			.iter()
			.map(|cell| cell.get())
			.position(|byte| byte == 0)
			.and_then(|length| {
				ptr.get_utf8_str(memory, length as u32)
					.map(|s| s.into_owned())
			})
	}
}
