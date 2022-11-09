use super::{
	CompileSnafu, Error, ExportSnafu, InstantiationSnafu, LockSnafu, ModuleSnafu, Runtime,
	RuntimeSnafu,
};
use crate::common::Address;

use std::{
	cell::RefCell,
	convert::identity,
	fmt::Display,
	num::NonZeroU32,
	ops::{Deref, DerefMut},
	sync::{Arc, RwLock},
};

use parking_lot::ReentrantMutex;
use snafu::{NoneError, ResultExt};
use wasm_bindgen::prelude::wasm_bindgen;
use wasmer::{
	FromToNativeWasmType, Function, FunctionEnv, FunctionEnvMut, Instance, Memory32, MemoryView,
	Module, Store, Type, Val, Value, WasmPtr,
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
	children: Arc<RwLock<Vec<Option<Arc<Actor>>>>>,

	// Process slots that have been freed, and are available for use
	free_slots: Arc<RwLock<Vec<Address>>>,
}

/// A handle to the runtime exposed to runtime API methods allowing
/// address introspection.
#[derive(Clone)]
pub struct RtContext(Rt, Address);

/// The source code of an actor, and its current state.
struct Actor {
	module: Arc<ReentrantMutex<Module>>,
	instance: Instance,
	store: ReentrantMutex<RefCell<Store>>,
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
	fn do_log_safe(env: FunctionEnvMut<(Address, Rt)>, msg: i32) -> Option<()> {
		let children = env.data().1.children.read().ok()?;
		let logging_actor = children.get(env.data().0 as usize).map(Option::as_ref)??;
		let memory = logging_actor.instance.exports.get_memory("memory").ok()?;

		// Get the memory of the module calling log, and read the message they
		// want to log from memory
		let memory = memory.view(&env);
		let msg = <WasmPtr<u8, Memory32> as FromToNativeWasmType>::from_native(msg)
			.read_utf8_string_with_nul(&memory)
			.ok()?;

		log(msg.as_str());
		Some(())
	}

	fn log_safe(env: FunctionEnvMut<(Address, Rt)>, msg: i32) {
		Self::do_log_safe(env, msg).unwrap();
	}

	fn do_send_message(
		&self,
		env: FunctionEnvMut<(Address, Rt)>,
		from: Address,
		addr: Address,
		msg_name_buf: i32,
		msg_buf: i32,
	) -> Option<()> {
		let children = self.children.read().ok()?;

		let sending_actor = children.get(env.data().0 as usize).map(Option::as_ref)??;
		let memory = sending_actor
			.instance
			.exports
			.get_memory("memory")
			.ok()?
			.view(&env);

		let msg_name = {
			// Get the message name from the sender, and call the receiver's handler
			let mut msg_name =
				<WasmPtr<u8, Memory32> as FromToNativeWasmType>::from_native(msg_name_buf)
					.read_utf8_string_with_nul(&memory)
					.ok()?;
			msg_name.insert_str(0, "handle_");

			msg_name
		};

		log(&format!(
			"sending message {:?} {:?} {:?}",
			from, addr, msg_name,
		));

		let recv = children.get(addr as usize).map(Option::as_ref).flatten()?;
		let recv_store = recv.store.lock();

		log("137");

		let handler = recv.instance.exports.get_function(msg_name.as_str()).ok()?;

		// Deref args expected by the handler from the message buffer
		let (args, _) = handler.ty(recv_store.borrow_mut().deref_mut()).params()[1..]
			.iter()
			.fold(
				([Value::I32(from as i32)].to_vec(), 0),
				|(mut accum, pos), arg| {
					let arg_size = type_size(arg);

					let parse_arg = |arg_val: Vec<i32>, view: &MemoryView| {
						let bytes = arg_val
							.into_iter()
							.map(|cell: i32| {
								<WasmPtr<u8, Memory32> as FromToNativeWasmType>::from_native(cell)
									.read(&view)
							})
							.collect::<Result<Vec<u8>, _>>()
							.ok()?;

						match arg {
							Type::I32 => {
								Some(Value::I32(i32::from_le_bytes(bytes.try_into().ok()?)))
							}
							Type::I64 => {
								Some(Value::I64(i64::from_le_bytes(bytes.try_into().ok()?)))
							}
							Type::F32 => {
								Some(Value::F32(f32::from_le_bytes(bytes.try_into().ok()?)))
							}
							Type::F64 => {
								Some(Value::F64(f64::from_le_bytes(bytes.try_into().ok()?)))
							}
							Type::V128 => {
								Some(Value::V128(u128::from_le_bytes(bytes.try_into().ok()?)))
							}

							// The only way for actors to communicate is via
							// message-passing. No cross-module dereferencing of pointers
							Type::ExternRef | Type::FuncRef => None,
						}
					};

					if let Some(arg_val) = parse_arg(
						(0..(arg_size / 8))
							.map(|i| msg_buf + (pos + i as i32))
							.collect::<Vec<i32>>(),
						&memory,
					) {
						accum.push(arg_val);

						(accum, pos + arg_size as i32)
					} else {
						(accum, pos)
					}
				},
			);

		log("197");

		handler
			.call(recv_store.borrow_mut().deref_mut(), args.as_slice())
			.unwrap();

		log("handled");

		Some(())
	}

	fn do_spawn_actor(&self, spawner: Option<Address>, addr: Address) -> Option<Address> {
		log("203");
		let child = {
			let children = self.children.read().ok()?;

			children.get(addr as usize)?.as_ref()?.clone()
		};

		let src = &child.src;

		log("209");

		self.spawn(spawner, src, false).ok()
	}

	fn send_message(
		env: FunctionEnvMut<(Address, Rt)>,
		addr: Address,
		msg_name_buf: i32,
		msg_buf: i32,
	) {
		let from = env.data().0;

		// Ensures that provided addresses aren't the root service
		if let Some((from, addr)) = NonZeroU32::new(from).zip(NonZeroU32::new(addr)) {
			let rt = env.data().1.clone();

			rt.do_send_message(env, from.get(), addr.get(), msg_name_buf, msg_buf);
		}
	}

	fn spawn_actor(env: FunctionEnvMut<(Address, Rt)>, addr: Address) -> Address {
		log(&format!(
			"{} requested to spawn a copy of {}",
			env.data().0,
			addr
		));

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
		log("255");
		let mut slots = self
			.free_slots
			.write()
			.map_err(|_| NoneError)
			.context(LockSnafu)?;
		log("261");
		log("here");
		let lock = self.children.try_write();
		log(&format!("{:?}", lock.is_ok()));
		let mut children = lock.map_err(|_| NoneError).context(LockSnafu)?;

		log("268");

		// Use the most recently freed process ID as the ID of the new process,
		// or use the index of a new slot
		let slot: Address = if let Some(free_slot) = slots.pop() {
			free_slot
		} else {
			log("275");
			let new_slot =
				TryInto::<u32>::try_into(children.len()).map_err(|_| Error::NoFreeAddrs)?;
			children.push(None);
			log("279");

			NonZeroU32::new(new_slot)
				.ok_or(Error::NoFreeAddrs)
				.map(NonZeroU32::get)?
		};

		log("281");

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

		log("319");

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
			module: Arc::new(ReentrantMutex::new(module)),
			src: src.as_ref().to_vec(),
			store: ReentrantMutex::new(RefCell::new(store)),
		};

		log("339");

		// Addresses are just indices in the set of current children
		// (ID's reused if a slot is freed)
		children[slot as usize] = Some(Arc::new(actor));

		Ok(slot)
	}

	fn impulse(
		&self,
		msg_name: impl AsRef<str> + Display,
		params: impl Deref<Target = [Val]>,
	) -> Vec<Result<(), Error>> {
		let handler_name = format!("handle_{}", msg_name);

		// Obtain an immutable copy of the children of the runtime
		let children = if let Some(children) = self.children.read().ok().map(|children| {
			children
				.iter()
				.map(Option::as_ref)
				.filter_map(identity)
				.map(|child| child.clone())
				.collect::<Vec<Arc<Actor>>>()
		}) {
			children
		} else {
			return Vec::new();
		};

		children
			.into_iter()
			.map(|ref child| {
				child
					.instance
					.exports
					.get_function(&handler_name)
					.context(ExportSnafu)
					.context(ModuleSnafu)
					.and_then(|ref handler| {
						let lock = child.store.lock();

						// Reallocate args with the sender being the master process
						let mut proper_params = params.to_vec();
						proper_params.push(Value::I32(0));

						let mut locked = lock.borrow_mut();

						handler
							.call(locked.deref_mut(), proper_params.as_slice())
							.map(|_| ())
							.context(RuntimeSnafu)
							.context(ModuleSnafu)
					})
			})
			.collect::<Vec<_>>()
	}
}
