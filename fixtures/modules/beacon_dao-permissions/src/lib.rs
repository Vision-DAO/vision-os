pub use vision_derive::beacon_dao_allocator;

use vision_derive::with_bindings;
use vision_utils::types::{
	Address, Callback, DISPLAY_MANAGER_ADDR, EXIT_FAILURE, EXIT_SUCCESS, PERM_AGENT_ADDR,
};

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
pub extern "C" fn handle_register_permission(
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

/// Gets the description of the permission.
#[no_mangle]
#[with_bindings]
pub extern "C" fn handle_get_permission(
	from: Address,
	name: String,
	callback: Callback<Option<String>>,
) {
	fn desc(name: String) -> Option<String> {
		let lock = PERMISSIONS.read().ok()?;
		let desc = lock.get(&name)?;

		Some(desc.0.clone())
	}

	callback.call(desc(name));
}

/// Checks that the actor has the identified permission.
#[no_mangle]
#[with_bindings]
pub extern "C" fn handle_has_permission(
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

/// Registers that a user has a permission if the message is being sent by the permission agent.
#[no_mangle]
#[with_bindings]
pub fn handle_set_permission(
	from: Address,
	actor: Address,
	permission: String,
	callback: Callback<u32>,
) {
	// Check that the user has permission to record permissions
	if from != PERM_AGENT_ADDR {
		callback.call(EXIT_FAILURE);

		return;
	}

	// Set the designated user as having the designated
	// permission in the permission registry
	let mut lock = if let Ok(lock) = PERMISSIONS.write() {
		lock
	} else {
		callback.call(EXIT_FAILURE);

		return;
	};

	let perm = if let Some(lock) = lock.get_mut(&permission) {
		lock
	} else {
		callback.call(EXIT_FAILURE);

		return;
	};
	perm.1.insert(actor);

	callback.call(EXIT_SUCCESS);
}
