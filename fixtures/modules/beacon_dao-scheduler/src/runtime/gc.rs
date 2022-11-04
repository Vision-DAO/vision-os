use super::{
	CompileSnafu, Error, ExportSnafu, InstantiationSnafu, LockSnafu, ModuleSnafu, Runtime,
	RuntimeSnafu,
};
use crate::common::Address;

use std::{
	convert::identity,
	fmt::Display,
	num::NonZeroU32,
	ops::{Deref, DerefMut},
	sync::{Arc, RwLock},
};

use snafu::{NoneError, ResultExt};
use wasm_bindgen::prelude::wasm_bindgen;
use wasmer::{
	Function, FunctionEnv, FunctionEnvMut, Instance, Module, RuntimeError, Store, Type, Val, Value,
	WasmPtr, WasmRef,
};

#[wasm_bindgen]
extern "C" {
	#[wasm_bindgen(js_namespace = console)]
	fn log(s: &str);
}

/// A naive garbage-collected implementation of the VVM scheduler.
#[derive(Clone)]
pub struct Rt {
	// Current running processes.
	children: Arc<RwLock<Vec<Option<Actor>>>>,

	// Process slots that have been freed, and are available for use
	free_slots: Arc<RwLock<Vec<Address>>>,
}

/// A handle to the runtime exposed to runtime API methods allowing
/// address introspection.
#[derive(Clone)]
pub struct RtContext(Rt, Address);

/// The source code of an actor, and its current state.
struct Actor {
	module: Arc<RwLock<Module>>,
	instance: Instance,
	store: RwLock<Store>,
	src: Vec<u8>,
}

// This is fine because modules, which are usually !Send + !Sync, are wrapped in a lock
// Citation: https://doc.rust-lang.org/nomicon/send-and-sync.html
unsafe impl Send for Actor {}
unsafe impl Sync for Actor {}

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
			children: Arc::new(RwLock::new(vec![None])),
			free_slots: Arc::new(RwLock::new(Vec::new())),
		}
	}
}

impl Rt {
	fn do_log_safe(env: FunctionEnvMut<(Address, Rt)>, msg: WasmPtr<u8>) -> Option<()> {
		// Get the memory of the module calling log, and read the message they
		// want to log from memory
		let children = env.data().1.children.read().ok()?;
		let self_module = children
			.get(env.data().0 as usize)
			.map(Option::as_ref)
			.flatten()?;
		let self_store = self_module.store.read().ok()?;

		let msg = msg
			.read_utf8_string_with_nul(
				&self_module
					.instance
					.exports
					.get_memory("memory")
					.ok()?
					.view(self_store.deref()),
			)
			.ok()?;

		log(msg.as_str());

		Some(())
	}

	fn log_safe(env: FunctionEnvMut<(Address, Rt)>, msg: WasmPtr<u8>) {
		Self::do_log_safe(env, msg).unwrap();
	}

	fn do_send_message(
		&self,
		from: Address,
		addr: Address,
		msg_name_buf: WasmPtr<u8>,
		msg_buf: WasmPtr<u8>,
	) -> Option<()> {
		let children = self.children.read().ok()?;

		let msg_name = {
			// Get the message name from the sender, and call the receiver's handler
			let sender = children.get(from as usize).map(Option::as_ref).flatten()?;
			let sender_store = sender.store.read().ok()?;

			log("5");

			let mut msg_name = msg_name_buf
				.read_utf8_string_with_nul(
					&sender
						.instance
						.exports
						.get_memory("memory")
						.ok()?
						.view(sender_store.deref()),
				)
				.ok()?;
			log("6");
			msg_name.insert_str(0, "handle_");
			log("7");

			msg_name
		};

		log(&format!(
			"sending message {:?} {:?} {:?}",
			from, addr, msg_name,
		));

		let recv = children.get(addr as usize).map(Option::as_ref).flatten()?;
		let mut recv_store = recv.store.write().ok()?;

		let handler = recv.instance.exports.get_function(msg_name.as_str()).ok()?;

		// Deref args expected by the handler from the message buffer
		let (args, _) = handler.ty(recv_store.deref()).params()[1..].iter().fold(
			([Value::I32(from as i32)].to_vec(), 0),
			|(mut accum, pos), arg| {
				let arg_size = type_size(arg);

				let parse_arg = |arg_val: Vec<WasmRef<'_, u8>>| {
					let bytes = arg_val
						.iter()
						.map(|cell| cell.read())
						.collect::<Result<Vec<u8>, _>>()
						.ok()?;

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

				if let Some(arg_val) =
					recv.instance
						.exports
						.get_memory("memory")
						.ok()
						.and_then(|mem| {
							let view = mem.view(recv_store.deref());

							(0..(arg_size / 8))
								.map(|i| {
									msg_buf
										.add_offset(pos + i as u32)
										.map(|ptr| ptr.deref(&view))
								})
								.collect::<Result<Vec<WasmRef<'_, u8>>, _>>()
								.ok()
								.and_then(parse_arg)
						}) {
					accum.push(arg_val);

					(accum, pos + arg_size)
				} else {
					(accum, pos)
				}
			},
		);

		handler
			.call(recv_store.deref_mut(), args.as_slice())
			.unwrap();

		Some(())
	}

	fn do_spawn_actor(&self, spawner: Option<Address>, addr: Address) -> Option<Address> {
		let children = self.children.read().ok()?;
		let child = children.get(addr as usize)?.as_ref()?;

		let src = &child.src;

		self.spawn(spawner, src, false).ok()
	}

	fn send_message(
		env: FunctionEnvMut<(Address, Rt)>,
		addr: Address,
		msg_name_buf: WasmPtr<u8>,
		msg_buf: WasmPtr<u8>,
	) {
		let from = env.data().0;

		// Ensures that provided addresses aren't the root service
		if let Some((from, addr)) = NonZeroU32::new(from).zip(NonZeroU32::new(addr)) {
			env.data()
				.1
				.do_send_message(from.get(), addr.get(), msg_name_buf, msg_buf);
		}
	}

	fn spawn_actor(env: FunctionEnvMut<(Address, Rt)>, addr: Address) -> Address {
		env.data()
			.1
			.do_spawn_actor(Some(env.data().0), addr)
			.unwrap_or(0)
	}

	fn address(env: FunctionEnvMut<Address>) -> Address {
		*env.data()
	}
}

impl Runtime for Rt {
	fn spawn(
		&self,
		spawner: Option<Address>,
		src: impl AsRef<[u8]>,
		privileged: bool,
	) -> Result<Address, Error> {
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
				TryInto::<u32>::try_into(children.len()).map_err(|_| Error::NoFreeAddrs)?;
			children.push(None);

			NonZeroU32::new(new_slot)
				.ok_or(Error::NoFreeAddrs)
				.map(NonZeroU32::get)?
		};

		let mut store = Store::default();
		let module = Module::new(&store, src.as_ref())
			.map_err(|_| NoneError)
			.context(CompileSnafu)
			.context(ModuleSnafu)?;

		// Create methods for the WASM module that allow spawning and sending
		let env = FunctionEnv::new(&mut store, (slot, self.clone()));
		let send_message_fn = Function::new_typed_with_env(&mut store, &env, Self::send_message);
		let spawn_actor_fn = Function::new_typed_with_env(&mut store, &env, Self::spawn_actor);
		let env = FunctionEnv::new(&mut store, slot);
		let address_fn = Function::new_typed_with_env(&mut store, &env, Self::address);
		let env = FunctionEnv::new(&mut store, (slot, self.clone()));
		let imports = if privileged {
			wasmer::imports! {
				"env" => {
					"send_message" => send_message_fn,
					"spawn_actor" => spawn_actor_fn,

					// Gets the address of the calling actor
					"address" => address_fn,
					"print" => Function::new_typed_with_env(&mut store, &env, Self::log_safe),
				},
			}
		} else {
			wasmer::imports! {
				"env" => {
					"send_message" => send_message_fn,
					"spawn_actor" => spawn_actor_fn,

					// Gets the address of the calling actor
					"address" => address_fn,
				},
			}
		};

		let instance = Instance::new(&mut store, &module, &imports)
			.context(InstantiationSnafu)
			.context(ModuleSnafu)?;

		if let Ok(init) = instance.exports.get_function("init") {
			if let Some(addr) = spawner {
				init.call(&mut store, &[Value::I32(addr as i32)]).unwrap();
			}
		}

		// Initialize an actor for the module, and call its initializer
		let actor = Actor {
			instance,
			module: Arc::new(RwLock::new(module)),
			src: src.as_ref().to_vec(),
			store: RwLock::new(store),
		};

		// Addresses are just indices in the set of current children
		// (ID's reused if a slot is freed)
		children[slot as usize] = Some(actor);

		Ok(slot)
	}

	fn impulse(
		&self,
		msg_name: impl AsRef<str> + Display,
		params: impl Deref<Target = [Val]>,
	) -> Vec<Result<(), Error>> {
		let handler_name = format!("handle_{}", msg_name);

		if let Ok(ref children) = self.children.read() {
			children
				.iter()
				.map(Option::as_ref)
				.filter_map(identity)
				.map(|ref child| {
					child
						.instance
						.exports
						.get_function(&handler_name)
						.context(ExportSnafu)
						.context(ModuleSnafu)
						.and_then(|ref handler| {
							let mut lock = child
								.store
								.write()
								.map_err(|_| NoneError)
								.context(LockSnafu)?;

							// Reallocate args with the sender being the master process
							let mut proper_params = params.to_vec();
							proper_params.push(Value::I32(0));

							handler
								.call(lock.deref_mut(), proper_params.as_slice())
								.map(|_| ())
								.context(RuntimeSnafu)
								.context(ModuleSnafu)
						})
				})
				.collect::<Vec<_>>()
		} else {
			Vec::new()
		}
	}
}
