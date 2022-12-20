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

	#[snafu(display("A serialization operation failed"))]
	SerializationError,
}

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
pub enum WasmError {
	InstantiationError { source: InstantiationError },
	RuntimeError { source: RuntimeError },
	ExportError { source: ExportError },
	CompileError,
}
