use super::{CompileSnafu, Error, InstantiationSnafu, LockSnafu, ModuleSnafu, Runtime};
use crate::common::{memory::get_utf8_string_with_nul, Address};

use std::{
	fmt::Display,
	num::NonZeroU32,
	ops::Deref,
	sync::{Arc, RwLock},
};

use snafu::{NoneError, ResultExt};
use wasmer::{
	Array, Function, Instance, Module, RuntimeError, Store, Type, Val, Value, WasmCell, WasmPtr,
	WasmerEnv,
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
		Type::FuncRef => 32,
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

impl Rt {
	fn do_send_message(
		&self,
		from: Address,
		addr: Address,
		msg_name_buf: WasmPtr<u8, Array>,
		msg_buf: WasmPtr<u8, Array>,
	) -> Option<()> {
		let children = self.children.read().ok()?;

		// Get the message name from the sender, and call the receiver's handler
		let sender = children.get(from as usize).map(Option::as_ref).flatten()?;
		let recv = children.get(addr as usize).map(Option::as_ref).flatten()?;

		let mut msg_name = get_utf8_string_with_nul(
			msg_name_buf,
			sender.instance.exports.get_memory("memory").ok()?,
		)?;
		msg_name.insert_str(0, "handle_");

		let handler = recv.instance.exports.get_function(msg_name.as_str()).ok()?;

		// Deref args expected by the handler from the message buffer
		let (mut args, _) = handler.ty().params()[1..].iter().fold(
			([Value::I32(from as i32)].to_vec(), 0),
			|(mut accum, pos), arg| {
				let arg_size = type_size(arg);

				let parse_arg = |arg_val: Vec<WasmCell<'_, u8>>| {
					let bytes = arg_val.iter().map(|cell| cell.get()).collect::<Vec<u8>>();

					match arg {
						Type::I32 => Some(Value::I32(i32::from_le_bytes(bytes.try_into().ok()?))),
						Type::I64 => Some(Value::I64(i64::from_le_bytes(bytes.try_into().ok()?))),
						Type::F32 => Some(Value::F32(f32::from_le_bytes(bytes.try_into().ok()?))),
						Type::F64 => Some(Value::F64(f64::from_le_bytes(bytes.try_into().ok()?))),
						Type::V128 => {
							Some(Value::V128(u128::from_le_bytes(bytes.try_into().ok()?)))
						}

						// The only way for actors to communicate is via
						// message-passing. No cross-module dereferencing of pointers
						Type::ExternRef | Type::FuncRef => None,
					}
				};

				if let Some(arg_val) = recv
					.instance
					.exports
					.get_memory("memory")
					.ok()
					.and_then(|mem| msg_buf.deref(mem, pos as u32, arg_size))
					.and_then(parse_arg)
				{
					accum.push(arg_val);

					(accum, pos + arg_size)
				} else {
					(accum, pos)
				}
			},
		);

		handler.call(args.as_slice());

		Some(())
	}

	fn send_message(
		env: &Rt,
		from: Address,
		addr: Address,
		msg_name_buf: WasmPtr<u8, Array>,
		msg_buf: WasmPtr<u8, Array>,
	) {
		// Ensures that provided addresses aren't the root service
		if let Some((from, addr)) = NonZeroU32::new(from).zip(NonZeroU32::new(addr)) {
			env.do_send_message(from.get(), addr.get(), msg_name_buf, msg_buf);
		}
	}

	fn spawn_actor(_env: &Rt, _addr: Address) -> Address {
		unimplemented!()
	}
}

impl Runtime for Rt {
	fn spawn(&self, src: impl AsRef<[u8]>) -> Result<Address, Error> {
		let mut slots = self
			.free_slots
			.write()
			.map_err(|_| NoneError)
			.context(LockSnafu)?;
		let mut children = self
			.children
			.write()
			.map_err(|_| NoneError)
			.context(LockSnafu)?;

		// Use the most recently freed process ID as the ID of the new process,
		// or use the index of a new slot
		let slot: Address = if let Some(free_slot) = slots.pop() {
			free_slot
		} else {
			let new_slot =
				TryInto::<u32>::try_into(children.len() + 1).map_err(|_| Error::NoFreeAddrs)?;
			children.push(None);

			NonZeroU32::new(new_slot)
				.ok_or(Error::NoFreeAddrs)
				.map(NonZeroU32::get)?
		};

		let store = Store::default();
		let module = Module::new(&store, src)
			.map_err(|_| NoneError)
			.context(CompileSnafu)
			.context(ModuleSnafu)?;

		// Create methods for the WASM module that allow spawning and sending
		let send_message_fn =
			Function::new_native_with_env(&store, self.clone(), Self::send_message);
		let spawn_actor_fn = Function::new_native_with_env(&store, self.clone(), Self::spawn_actor);
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
				.context(InstantiationSnafu)
				.context(ModuleSnafu)?,
			module,
		};

		if let Ok(init) = actor.instance.exports.get_function("init") {
			init.call(&[]);
		}

		// Addresses are just indices in the set of current children
		// (ID's reused if a slot is freed)
		children[slot as usize] = Some(actor);

		Ok(slot)
	}

	fn impulse(
		&self,
		msg_name: impl AsRef<str> + Display,
		params: impl Deref<Target = [Val]>,
	) -> Vec<Result<(), RuntimeError>> {
		let handler_name = format!("handle_{}", msg_name);

		if let Ok(ref children) = self.children.read() {
			children
				.iter()
				.filter_map(|c| c.as_ref())
				.map(move |child| child.instance.exports.get_function(&handler_name))
				.filter_map(Result::ok)
				.map(move |handler| {
					// Reallocate args with the sender being the master process
					let mut proper_params = params.to_vec();
					proper_params.push(Value::I32(0));

					handler.call(proper_params.as_slice()).map(|_| ())
				})
				.collect::<Vec<_>>()
		} else {
			Vec::new()
		}
	}
}
