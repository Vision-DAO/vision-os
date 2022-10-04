use super::{Error, Runtime, WasmSnafu};
use crate::common::Address;

use std::{
	error::Error as StdError,
	fmt::Display,
	iter,
	num::NonZeroU32,
	ops::Deref,
	sync::{Arc, RwLock},
};

use snafu::ResultExt;
use wasmer::{
	Array, Function, Instance, Module, RuntimeError, Store, Type, Val, Value, WasmPtr, WasmerEnv,
};

/// A naive garbage-collected implementation of the VVM scheduler.
#[derive(WasmerEnv, Clone)]
pub struct Rt {
	// Current running processes.
	children: Arc<RwLock<Vec<Option<Actor>>>>,

	// Process slots that have been freed, and are available for use
	free_slots: Arc<RwLock<Vec<Address>>>,
}

/// The source code of an actor, and its current state.
struct Actor {
	module: Module,
	instance: Instance,
	store: Store,
}

fn type_size(ty: impl Deref<Target = Type>) -> u32 {
	match *ty {
		Type::I32 => 32,
		Type::I64 => 64,
		Type::F32 => 32,
		Type::F64 => 64,
		Type::V128 => 128,
		Type::ExternRef => 32,
		Type::ExternRef => 32,
	}
}

impl Default for Rt {
	fn default() -> Self {
		Self {
			children: Arc::new(RwLock::new(Vec::new())),
			free_slots: Arc::new(RwLock::new(Vec::new())),
		}
	}
}

impl Runtime for Rt {
	fn send_message(env: &Rt, from: Address, addr: Address, msg_buf: WasmPtr<u8, Array>) {
		let handler_name = format!("handle_{}", msg_name);
		if let Some((handler, mem)) = env
			.children
			.read()
			.ok()
			.and_then(|children| children.get(addr.get() as usize))
			.map(Option::as_ref)
			.flatten()
			.and_then(|actor: &Actor| {
				actor
					.instance
					.exports
					.get_function(handler_name.as_str())
					.ok()
					.zip(actor.instance.exports.get_memory("memory").ok())
			}) {
			// Deref args expected by the handler from the message buffer
			let (mut args, _) = handler.ty().params()[1..].iter().fold(
				(&[Value::I32(from.get() as i32)].to_vec(), 0),
				|(accum, pos), arg| {
					if let Some(arg) = msg_buf
						.deref(mem, pos as u32, type_size(arg))
						.and_then(|data| data.get(0))
					// TODO: Build out parameters from raw bytes
					{
						accum.push(arg);

						(accum, pos + type_size(arg))
					} else {
						(accum, pos)
					}
				},
			);

			handler.call(args.as_slice());
		}
	}

	fn spawn_actor(env: &Rt, addr: Address) -> Address {
		unimplemented!()
	}

	fn spawn(&self, src: impl AsRef<[u8]>) -> Result<Address, Error> {
		let slots = self.free_slots.write()?;
		let children = self.children.write()?;

		// Use the most recently freed process ID as the ID of the new process,
		// or use the index of a new slot
		let slot: Address = if let Some(free_slot) = slots.pop() {
			NonZeroU32::new(free_slot)?
		} else {
			let new_slot =
				TryInto::<u32>::try_into(children.len() + 1).map_err(|_| Error::NoFreeAddrs)?;
			children.push(None);

			NonZeroU32::new(new_slot)?
		};

		let store = Store::default();
		let module = Module::new(&store, src)
			.map_err(|e| Box::new(e) as Box<dyn StdError>)
			.context(WasmSnafu)?;

		// Create methods for the WASM module that allow spawning and sending
		let send_message_fn = Function::new_native_with_env(&store, self, send_message_fn);
		let spawn_actor_fn = Function::new_native_with_env(&store, self, spawn_actor_fn);
		let imports = wasmer::imports! {
			"" => {
				"send_message" => send_message_fn,
				"spawn_actor" => spawn_actor_fn,
			},
		};

		// Initialize an actor for the module, and call its initializer
		let actor = Actor {
			store,
			instance: Instance::new(&module, &imports)
				.map_err(|e| Box::new(e) as Box<dyn StdError>)
				.context(WasmSnafu)?,
			module,
		};

		if let Ok(init) = actor.instance.exports.get_function("init") {
			init.call(&[]);
		}

		// Addresses are just indices in the set of current children
		// (ID's reused if a slot is freed)
		children[slot.get() as usize] = Some(actor);

		Ok(slot)
	}

	fn impulse<'a>(
		&'a self,
		msg_name: impl AsRef<str> + Display,
		params: &'a [Val],
	) -> Box<dyn Iterator<Item = Result<(), RuntimeError>> + 'a> {
		let handler_name = format!("handle_{}", msg_name);

		// Reallocate args with the sender being the master process
		let proper_params = params.to_vec();
		proper_params.push(Value::I32(0));

		if let Ok(children) = self.children.read() {
			Box::new(
				children
					.iter()
					.filter_map(|c| c.as_ref())
					.map(move |child| child.instance.exports.get_function(&handler_name))
					.filter_map(Result::ok)
					.map(|handler| handler.call(proper_params.as_slice()).map(|_| ())),
			)
		} else {
			Box::new(iter::empty())
		}
	}
}
