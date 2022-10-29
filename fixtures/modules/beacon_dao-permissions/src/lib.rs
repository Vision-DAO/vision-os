use beacon_dao_scheduler::common::Address;

/// Services, and their authorized tokens.
#[wasm_bindgen]
static CHANNELS: RwLock<HashMap<Address, u128>>;
