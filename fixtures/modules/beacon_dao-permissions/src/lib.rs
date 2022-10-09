use beacon_dao_scheduler::common::Address;

/// Services, and their authorized tokens.
#[wasm_bindgen]
pub static CHANNELS: RwLock<HashMap<Address, u128>>;

#[wasm_bindgen]
/// Starts advertising the service for usage by consumers.
pub fn handle_announce_service(sender: Address, passive: bool, name: impl AsRef<str>) {
	let registry = CHANNELS.write().unwrap();
}
