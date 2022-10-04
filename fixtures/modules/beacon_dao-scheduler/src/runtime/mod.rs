pub mod gc;

use crate::common::Address;
use snafu::Snafu;
use wasmer::{Array, RuntimeError, Val, WasmPtr};

use std::{
	error::Error as StdError,
	fmt::{Debug, Display},
};

/// Any error encountered by the VVM runtime.
#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
	#[snafu(display("No addresses were available for allocation"))]
	NoFreeAddrs,

	#[snafu(display("The WebAssembly module encountered an error: {source}"))]
	WasmError { source: Box<dyn StdError> },
}

/// A Vision Virtual Machine scheduler.
pub trait Runtime {
	/// Creates a new actor with the specified module code, returning the
	/// address identifying the newly spawned actor. Also calls the
	/// initialization function of the actor.
	fn spawn(&self, module: impl AsRef<[u8]>) -> Result<Address, Error>;

	/// Sends the message at msg_buf to the actor at addr, returning nothing.
	fn send_message(rt: &Self, from: Address, addr: Address, msg_buf: WasmPtr<u8, Array>);

	/// Creates a duplicate of the actor at the address.
	fn spawn_actor(rt: &Self, addr: Address) -> Address;

	/// Sends a simulated message to all actors that implement handlers for it.
	fn impulse<'a>(
		&'a self,
		msg_name: impl AsRef<str> + Display,
		params: &'a [Val],
	) -> Box<dyn Iterator<Item = Result<(), RuntimeError>> + 'a>;
}
