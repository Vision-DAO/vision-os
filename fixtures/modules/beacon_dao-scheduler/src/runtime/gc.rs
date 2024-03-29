use super::{
	CompileSnafu, Error, InstantiationSnafu, LockSnafu, ModuleSnafu, RuntimeSnafu, WasmError,
};
use crate::common::Address;

use std::{
	collections::HashMap,
	fmt::Display,
	mem,
	num::NonZeroU32,
	ops::{Deref, DerefMut},
	sync::{Arc, RwLock},
};

use super::api::log;
use js_sys::{Array, JsString, JSON};
use snafu::{NoneError, ResultExt};
use wasmer::{
	Extern, FromToNativeWasmType, Function, FunctionEnv, FunctionEnvMut, FunctionType, Instance,
	Memory32, MemoryView, Module, Store, Type, Value, WasmPtr,
};

type Call = Vec<Value>;
type Mailbox = HashMap<String, Vec<Call>>;

pub(crate) struct Usps {
	boxes: Vec<Mailbox>,
	n_queued: usize,
}

impl Usps {
	fn new(n_queued: usize) -> Self {
		Self {
			boxes: (0..=n_queued).map(|_| HashMap::new()).collect(),
			n_queued,
		}
	}

	fn send_to(&mut self, to: Address, msg_name: String, args: Vec<Value>) -> Result<(), Error> {
		self.boxes
			.get_mut(to as usize)
			.ok_or(Error::InvalidAddressError)?
			.entry(msg_name)
			.or_default()
			.push(args);

		self.n_queued += 1;

		Ok(())
	}

	fn push(&mut self) {
		self.boxes.push(HashMap::new());
	}

	fn drain(&mut self) -> Vec<Mailbox> {
		self.n_queued = 0;
		let new_boxes = self
			.boxes
			.iter()
			.map(|_| HashMap::new())
			.collect::<Vec<_>>();

		mem::replace(&mut self.boxes, new_boxes)
	}
}

/// A naive garbage-collected implementation of the VVM scheduler.
#[derive(Clone)]
pub struct Rt {
	// Current running processes.
	pub(crate) children: Arc<RwLock<Vec<Option<Arc<Actor>>>>>,

	// Process slots that have been freed, and are available for use
	pub(crate) free_slots: Arc<RwLock<Vec<Address>>>,

	// Queued messages for sending to handlers per actor
	pub(crate) mailboxes: Arc<RwLock<Usps>>,
}

/// A handle to the runtime exposed to runtime API methods allowing
/// address introspection.
#[derive(Clone)]
pub struct RtContext(Rt, Address);

/// The source code of an actor, and its current state.
pub(crate) struct Actor {
	pub(crate) module: Arc<RwLock<Module>>,
	pub(crate) instance: Instance,
	pub(crate) store: Arc<RwLock<Store>>,
	pub(crate) src: Vec<u8>,
	pub(crate) abi: Arc<RwLock<HashMap<String, FunctionType>>>,
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
			mailboxes: Arc::new(RwLock::new(Usps::new(0))),
		}
	}
}

impl Rt {
	fn view_children(&self) -> Vec<Option<Arc<Actor>>> {
		// Obtain an immutable copy of the children of the runtime
		if let Some(children) = self.children.read().ok().map(|children| {
			children
				.iter()
				.map(Option::as_ref)
				.map(|child| child.cloned())
				.collect::<Vec<Option<Arc<Actor>>>>()
		}) {
			children
		} else {
			Vec::new()
		}
	}

	fn do_send_message(
		&self,
		env: FunctionEnvMut<(Address, Rt)>,
		from: Address,
		addr: Address,
		msg_name_buf: i32,
		msg_buf: i32,
	) -> Option<()> {
		let sending_actor = {
			let children = self.children.read().ok()?;
			children
				.get(env.data().0 as usize)
				.map(Option::as_ref)??
				.clone()
		};

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

		let recv = {
			let children = self.children.read().ok()?;
			children
				.get(addr as usize)
				.and_then(Option::as_ref)?
				.clone()
		};
		let abis = recv.abi.read().ok()?;
		let abi = abis.get(msg_name.as_str())?;

		// Deref args expected by the handler from the message buffer
		let (args, _) = abi.params()[1..].iter().fold(
			([Value::I32(from as i32)].to_vec(), 0),
			|(mut accum, pos), arg| {
				let arg_size = type_size(arg);
				let parse_arg = |arg_val: Vec<i32>, view: &MemoryView| {
					let bytes = arg_val
						.into_iter()
						.map(|cell: i32| {
							<WasmPtr<u8, Memory32> as FromToNativeWasmType>::from_native(cell)
								.read(view)
						})
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

				if let Some(arg_val) = parse_arg(
					(0..(arg_size / 8))
						.map(|i| msg_buf + (pos + i as i32))
						.collect::<Vec<i32>>(),
					&memory,
				) {
					accum.push(arg_val);

					(accum, pos + (arg_size / 8) as i32)
				} else {
					(accum, pos)
				}
			},
		);

		env.data()
			.1
			.mailboxes
			.write()
			.ok()?
			.send_to(addr, msg_name, args)
			.ok()?;

		Some(())
	}

	fn do_spawn_actor(&self, spawner: Option<Address>, addr: Address) -> Result<Address, Error> {
		let child = {
			let children = self
				.children
				.read()
				.map_err(|_| NoneError)
				.context(LockSnafu)?;

			children
				.get(addr as usize)
				.and_then(|child| child.clone())
				.ok_or(Error::InvalidAddressError)?
		};

		let src = &child.src;

		self.spawn(spawner, src, false)
	}

	fn do_spawn_actor_from(
		&self,
		spawner: Option<Address>,
		addr: Address,
	) -> Result<Address, Error> {
		let child = {
			let children = self
				.children
				.read()
				.map_err(|_| NoneError)
				.context(LockSnafu)?;

			children
				.get(addr as usize)
				.and_then(|child| child.clone())
				.ok_or(Error::InvalidAddressError)?
		};

		// Dynamically get the WASM src by reading from a cell
		let src = {
			let mut store = child.store.write().map_err(|_| Error::LockError)?;

			let len_fn = child
				.instance
				.exports
				.get_function("len_sync")
				.map_err(|_| NoneError)
				.context(CompileSnafu)
				.context(ModuleSnafu)?;
			let read_fn = child
				.instance
				.exports
				.get_function("read_sync")
				.map_err(|_| NoneError)
				.context(CompileSnafu)
				.context(ModuleSnafu)?;

			let len = len_fn
				.call(store.deref_mut(), &[])
				.context(RuntimeSnafu)
				.and_then(|ret| {
					ret.get(0)
						.and_then(|len| match len {
							Value::I32(v) => Some(v.clone()),
							_ => None,
						})
						.ok_or(WasmError::CompileError)
				})
				.context(ModuleSnafu)?;
			let mut src: Vec<u8> = Vec::new();

			for i in 0..len {
				src.push(
					read_fn
						.call(store.deref_mut(), &[Value::I32(i)])
						.context(RuntimeSnafu)
						.and_then(|ret| {
							ret.get(0)
								.and_then(|v| match v {
									Value::I32(v) => Some(*v as u8),
									_ => None,
								})
								.ok_or(WasmError::CompileError)
						})
						.context(ModuleSnafu)?,
				);
			}

			src
		};

		log(&format!("{:?}", src));
		log("328");

		self.spawn(spawner, src, false)
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
		env.data()
			.1
			.do_spawn_actor(Some(env.data().0), addr)
			.unwrap()
	}

	/// Spawns an actor from the webassembly module code in the indicated memory cell.
	fn spawn_actor_from(env: FunctionEnvMut<(Address, Rt)>, addr: Address) -> Address {
		env.data()
			.1
			.do_spawn_actor_from(Some(env.data().0), addr)
			.unwrap()
	}

	fn address(env: FunctionEnvMut<Address>) -> Address {
		*env.data()
	}

	/// Serializes the given JavaScript values into impulse-compatible WASM values (only copy types)
	/// Will perform an allocation if necessary
	pub fn impulse_js(
		&self,
		from: Option<Address>,
		to: Option<Address>,
		msg_name: &str,
		params: Array,
	) -> Result<(), Error> {
		let copy_params = params
			.iter()
			.map(|param| {
				// Try getting a copy type from the JS argument
				// Otherwise, serialize it as JSON, and pass it in as an
				// actor address
				param
					.as_f64()
					.and_then(|farg| {
						if farg.fract() == 0.0 {
							<u64 as TryInto<u32>>::try_into(farg as u64)
								.ok()
								.map(|val| Value::I32(val as i32))
								.or(Some(Value::I64(farg as i64)))
						} else {
							Some(Value::F64(farg))
						}
					})
					.ok_or(Error::SerializationError)
					// Serialize the fat fuck type
					.or_else(|_| {
						// Use JSON.stringify to get a JSON repr
						JSON::stringify(&param)
							.map(|farg| <JsString as Into<String>>::into(farg).into_bytes())
							.map_err(|_| Error::SerializationError)
							.and_then(|farg| {
								// Make a mock memory cell to allow other actors
								// to read this serialized info
								let mock_allocator = self
									.children
									.write()
									.map_err(|_| Error::LockError)?
									.get(8)
									.cloned()
									.flatten()
									.ok_or(Error::InvalidAddressError)?;

								let cell = mock_allocator
									.instance
									.exports
									.get_function("alloc")
									.map_err(|_| NoneError)
									.context(CompileSnafu)
									.context(ModuleSnafu)
									.and_then(|handler| {
										mock_allocator
											.store
											.write()
											.map_err(|_| Error::LockError)
											.and_then(|mut store| {
												handler
													.call(store.deref_mut(), &[])
													.context(RuntimeSnafu)
													.context(ModuleSnafu)
											})
									})
									.map(|ret| match ret[0] {
										Value::I32(v) => v,
										_ => panic!("Invalid declaration of mock spawn()"),
									})?;

								// Get a lock on the append() function
								// which will be used to fill up the cell with
								// serialized data
								let cell_actor = self
									.children
									.write()
									.map_err(|_| Error::LockError)?
									.get(cell as usize)
									.cloned()
									.flatten()
									.ok_or(Error::InvalidAddressError)?;
								let handler = cell_actor
									.instance
									.exports
									.get_function("append")
									.map_err(|_| NoneError)
									.context(CompileSnafu)
									.context(ModuleSnafu)?;
								cell_actor
									.store
									.write()
									.map_err(|_| Error::LockError)
									.and_then(|mut store| {
										// Write each byte of serialized
										// farg into the cell
										farg.into_iter().try_for_each(|byte| {
											handler
												.call(store.deref_mut(), &[Value::I32(byte as i32)])
												.context(RuntimeSnafu)
												.context(ModuleSnafu)
												.map(|_| ())
										})
									})?;

								Ok(Value::I32(cell))
							})
					})
			})
			.collect::<Result<Vec<Value>, Error>>()?;

		if let Some(to_addr) = to {
			self.impulse(from, to_addr, msg_name, copy_params)
		} else {
			self.impulse_all(from, msg_name, copy_params);
			Ok(())
		}
	}

	pub fn spawn(
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
		let lock = self.children.write();
		let mut children = lock.map_err(|_| NoneError).context(LockSnafu)?;

		// Use the most recently freed process ID as the ID of the new process,
		// or use the index of a new slot
		let slot: Address = if let Some(free_slot) = slots.pop() {
			free_slot
		} else {
			let mut mailboxes = self
				.mailboxes
				.write()
				.map_err(|_| NoneError)
				.context(LockSnafu)?;
			let new_slot =
				TryInto::<u32>::try_into(children.len()).map_err(|_| Error::NoFreeAddrs)?;
			children.push(None);
			mailboxes.push();

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
		let spawn_actor_from_fn =
			Function::new_typed_with_env(&mut store, &env, Self::spawn_actor_from);
		let env = FunctionEnv::new(&mut store, slot);
		let address_fn = Function::new_typed_with_env(&mut store, &env, Self::address);
		let env = FunctionEnv::new(&mut store, (slot, self.clone()));

		let imports = if privileged {
			wasmer::imports! {
				"env" => {
					"send_message" => send_message_fn,
					"spawn_actor" => spawn_actor_fn,
					"spawn_actor_from" => spawn_actor_from_fn,

					// Gets the address of the calling actor
					"address" => address_fn,
					"print" => Function::new_typed_with_env(&mut store, &env, Self::log_safe),
					"append_element" => Function::new_typed_with_env(&mut store, &env, Self::append_element_safe),
					"eval_js" => Function::new_typed_with_env(&mut store, &env, Self::eval_js_safe),
				},
			}
		} else {
			wasmer::imports! {
				"env" => {
					"send_message" => send_message_fn,
					"spawn_actor" => spawn_actor_fn,
					"spawn_actor_from" => spawn_actor_from_fn,

					// Gets the address of the calling actor
					"address" => address_fn,
					"print" => Function::new_typed(&mut store, |_: i32| {}),
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

		// Find the parameter and return types of all export of the module
		let abi = instance
			.exports
			.iter()
			.filter_map(|(name, export)| match export {
				Extern::Function(f) => Some((name.clone(), f.ty(&store))),
				_ => None,
			})
			.fold(HashMap::new(), |mut accum, (name, ty)| {
				accum.insert(name, ty);

				accum
			});

		// Initialize an actor for the module, and call its initializer
		let actor = Actor {
			instance,
			abi: Arc::new(RwLock::new(abi)),
			module: Arc::new(RwLock::new(module)),
			src: src.as_ref().to_vec(),
			store: Arc::new(RwLock::new(store)),
		};

		// Addresses are just indices in the set of current children
		// (ID's reused if a slot is freed)
		children[slot as usize] = Some(Arc::new(actor));
		self.impulse(spawner, slot, "init_async", &[][..]);

		Ok(slot)
	}

	pub fn impulse(
		&self,
		from: Option<Address>,
		to: Address,
		msg_name: impl AsRef<str> + Display,
		params: impl Deref<Target = [Value]>,
	) -> Result<(), Error> {
		let mut with_from_params = params.to_vec();
		with_from_params.insert(0, Value::I32(from.unwrap_or(0) as i32));

		self.mailboxes
			.write()
			.map_err(|_| Error::LockError)?
			.send_to(to, format!("handle_{}", msg_name), with_from_params)
	}

	pub fn impulse_all(
		&self,
		from: Option<Address>,
		msg_name: impl AsRef<str> + Display,
		params: impl Deref<Target = [Value]>,
	) {
		let n_children = if let Ok(lock) = self.children.read() {
			lock.len()
		} else {
			return;
		};

		for i in 0..n_children {
			if let Err(e) = self.impulse(from, i as u32, &msg_name, &params[..]) {
				log(&format!(
					"impulse handler error for message {} to process {}: {:?}",
					msg_name, i, e
				));
			}
		}
	}

	pub fn poll(&self) -> Result<(), Error> {
		// Handle all "real" (i.e., generated by actors within our system) messages
		// Continuously handle until no more real messages exist
		loop {
			if self
				.mailboxes
				.read()
				.map_err(|_| NoneError)
				.context(LockSnafu)?
				.n_queued == 0
			{
				break;
			}

			let queued = self
				.mailboxes
				.write()
				.map_err(|_| NoneError)
				.context(LockSnafu)?
				.deref_mut()
				.drain();
			let children = self.view_children();

			// Drain all queued messages on all actors
			for (mailbox, child) in queued.into_iter().zip(children.into_iter()).skip(1) {
				// Consider only queued messages for real children
				let child = if let Some(child) = child {
					child
				} else {
					continue;
				};

				// In order to call handlers for methods on the child,
				// the internal state must be mutated
				let mut lock = child
					.store
					.write()
					.map_err(|_| NoneError)
					.context(LockSnafu)?;

				// Send all messsages to the child that they have a handler for
				for (i, (msg_name, calls)) in mailbox.iter().enumerate() {
					let handler = if let Ok(handler) =
						child.instance.exports.get_function(msg_name.as_str())
					{
						handler
					} else {
						continue;
					};

					for args in calls {
						// Call the handler, and log an error if the call failed
						if let Err(e) = handler
							.call(lock.deref_mut(), args.as_slice())
							.map(|_| ())
							.context(RuntimeSnafu)
							.context(ModuleSnafu)
						{
							log(&format!(
								"real event handler error for message {} to process {}: {:?}",
								msg_name, i, e
							));
						}
					}
				}
			}
		}

		Ok(())
	}
}
