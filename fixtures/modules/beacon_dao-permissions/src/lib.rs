pub use vision_derive::beacon_dao_allocator;

use vision_derive::with_bindings;
use vision_utils::types::{Address, Callback};

use std::{
	collections::{HashMap, HashSet},
	sync::{Arc, RwLock},
};

/// Marks which actors have been given permission to use different capabilities.
lazy_static::lazy_static! {
	static ref PERMISSIONS: Arc<RwLock<HashMap<String, (String, HashSet<Address>)>>> =
		Arc::new(RwLock::new(HashMap::new()));
}

/// Registers a capability of the Vision OS that the user needs to consent to allowing.
#[no_mangle]
#[with_bindings]
pub fn handle_register_permission(
	from: Address,
	name: String,
	description: String,
	callback: Callback<u8>,
) {
	if let Ok(mut lock) = PERMISSIONS.write() {
		lock.entry(name).or_insert((description, HashSet::new()));

		callback.call(0);
	} else {
		callback.call(1);
	};
}

/// Checks that the actor has the identified permission.
#[no_mangle]
#[with_bindings]
pub fn handle_has_permission(
	from: Address,
	actor: Address,
	permission: String,
	callback: Callback<bool>,
) {
	if let Ok(lock) = PERMISSIONS.read() {
		callback.call(
			lock.get(&permission)
				.map(|actors_with_perm| actors_with_perm.1.contains(&actor))
				.unwrap_or(false),
		);
	} else {
		callback.call(false);
	}
}

#[no_mangle]
#[with_bindings]
pub fn handle_request_permission(from: Address, permission: String, callback: Callback<bool>) {
	// TODO: Implement this
	unimplemented!()
}
