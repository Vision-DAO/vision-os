use vision_derive::with_bindings;
use vision_utils::types::{Address, Callback};

use std::collections::{HashMap, HashSet};

/// Marks which actors have been given permission to use different capabilities.
static PERMISSIONS: Arc<RwLock<HashMap<String, (String, HashSet<Address>)>>>;

/// Registers a capability of the Vision OS that the user needs to consent to allowing.
#[no_mangle]
#[with_bindings]
pub fn handle_register_permission(
	from: Address,
	name: String,
	description: String,
	callback: Callback<u8>,
) {
	if let Some(lock) = PERMISSIONS.write() {
		lock.entry(name).or_insert((description, HashSet::new()));

		callback(0);
	} else {
		callback(1);
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
	if let Some(lock) = PERMISSIONS.read() {
		callback(
			lock.get(permission)
				.and_then(|actors_with_perm| actors_with_perm.has(actor))
				.unwrap_or(false),
		);
	} else {
		callback(false);
	}
}

#[no_mangle]
#[with_bindings]
pub fn handle_request_permission(from: Address, permission: String, callback: Callback<bool>) {
	// TODO: Implement this
	unimplemented!()
}
