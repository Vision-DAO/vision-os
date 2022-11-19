use vision_derive::with_bindings;
/*
TODO: Requires window manager to work.
use vision_utils::types::Address;

use std::collections::{HashMap, HashSet};

/// Marks which actors have been given permission to use different capabilities.
static PERMISSIONS: Arc<RwLock<HashMap<String, (String, HashSet<Address>)>>>;

/// Registers a capability of the Vision OS that the user needs to consent to allowing.
#[with_bindings]
pub fn handle_register_permission(from: Address, name: String, description: String) {
	PERMISSIONS
		.write()
		.unwrap()
		.entry(name)
		.or_insert((description, HashSet::new()));
}

/// Checks that the actor has the identified permission.
#[with_bindings]
pub fn handle_has_permission(from: Address, actor: Address, permission: String) -> bool {
	PERMISSIONS
		.read()
		.unwrap()
		.get(permission)
		.and_then(|actors_with_perm| actors_with_perm.has(actor))
		.unwrap_or(false)
}

#[with_bindings]
pub fn handle_request_permission(from: Address, permission: String) -> bool {}
*/
