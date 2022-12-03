use beacon_dao_test_pong::pong;
use vision_utils::{
	actor::address,
	types::{Address, Callback},
};

#[no_mangle]
pub extern "C" fn handle_ping(from: Address) {
	if address() != 6 {
		return;
	}

	pong(
		address() + 1,
		4,
		5,
		6,
		Callback::new(move |val| {
			extern "C" {
				fn print(s: i32);
			}

			let msg = std::ffi::CString::new(format!("ping {}", val)).unwrap();

			unsafe {
				print(msg.as_ptr() as i32);
			}
		}),
	);
}
