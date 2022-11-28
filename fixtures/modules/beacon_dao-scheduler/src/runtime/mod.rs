pub mod api;

/// Implements basic web API's for actors within the VVM.
pub mod gc;

use crate::common::Address;
use snafu::Snafu;
use wasmer::{ExportError, InstantiationError, RuntimeError, Value};

use std::{
	fmt::{Debug, Display},
	ops::Deref,
};

/// Any error encountered by the VVM runtime.
#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum Error {
	#[snafu(display("No addresses were available for allocation"))]
	NoFreeAddrs,

	#[snafu(display("The WebAssembly module encountered an error: {source}"))]
	ModuleError { source: WasmError },

	#[snafu(display("The server could not obtain a lock on a resource"))]
	LockError,

	#[snafu(display("No resource exists at the address"))]
	InvalidAddressError,

	#[snafu(display("No window exists for the runtime to bind to"))]
	MissingWindow,
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum WasmError {
	InstantiationError { source: InstantiationError },
	RuntimeError { source: RuntimeError },
	ExportError { source: ExportError },
	CompileError,
}

/// A Vision Virtual Machine scheduler.
pub trait Runtime {
	/// Creates a new actor with the specified module code, returning the
	/// address identifying the newly spawned actor. Also calls the
	/// initialization function of the actor.
	fn spawn(
		&self,
		spawner: Option<Address>,
		module: impl AsRef<[u8]>,
		privileged: bool,
	) -> Result<Address, Error>;

	/// Sends a simulated message to all actors that implement handlers for it.
	fn impulse(
		&self,
		msg_name: impl AsRef<str> + Display,
		params: impl Deref<Target = [Value]>,
	) -> Result<(), Error>;
}
