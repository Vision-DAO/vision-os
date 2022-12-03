use vision_derive::with_bindings;
use vision_utils::{
	actor::address,
	types::{Address, Callback},
};

#[no_mangle]
#[with_bindings]
pub extern "C" fn handle_pong(from: Address, val: u8, val2: u32, val3: u8, callback: Callback<u8>) {
	extern "C" {
		fn print(s: i32);
	}
	let msg = std::ffi::CString::new(format!("pong {} {} {}", val, val2, val3)).unwrap();
	unsafe {
		print(msg.as_ptr() as i32);
	}

	callback.call(val);
}
