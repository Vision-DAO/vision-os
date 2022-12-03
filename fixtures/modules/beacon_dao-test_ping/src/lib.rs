use vision_derive::with_bindings;
use vision_utils::{
	actor::address,
	types::{Address, Callback},
};

#[no_mangle]
#[with_bindings]
pub extern "C" fn handle_pong(from: Address, val: u8, callback: Callback<u8>) {
	extern "C" {
		fn print(s: i32);
	}
	let msg = std::ffi::CString::new(format!("{} 2", val)).unwrap();
	unsafe {
		print(msg.as_ptr() as i32);
	}

	callback.call(val);
}

#[no_mangle]
pub extern "C" fn handle_ping(from: Address) {
	if address() != 6 {
		return;
	}

	pong(
		address() + 1,
		1,
		Callback::new(move |val| {
			extern "C" {
				fn print(s: i32);
			}

			let msg = std::ffi::CString::new(format!("{}", val)).unwrap();

			unsafe {
				print(msg.as_ptr() as i32);
			}
		}),
	);
}
