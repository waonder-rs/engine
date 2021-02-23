use std::pin::Pin;
use std::task::{
	Context,
	Waker,
	Poll
};
use std::ops::DerefMut;
use std::sync::Arc;
use crossbeam_queue::SegQueue as AtomicQueue;
use parking_lot::Mutex;
use vulkano::{
	device::{
		Device,
		Queue
	},
	buffer::{
		BufferUsage,
		sys::{
			UnsafeBuffer,
			BufferCreationError,
			SparseLevel
		}
	},
	sync::{
		GpuFuture,
		Sharing
	},
	memory::{
		Content,
		MemoryRequirements,
		DedicatedAlloc,
		pool::{
			StdMemoryPool,
			MemoryPool,
			MemoryPoolAlloc,
			AllocLayout,
			MappingRequirement,
			AllocFromRequirementsFilter
		}
	},
	command_buffer::{
		AutoCommandBufferBuilder,
		CommandBuffer
	}
};

mod buffer;
mod pending;

pub use buffer::*;
pub use pending::*;

pub type AllocError = vulkano::memory::DeviceMemoryAllocError;

pub enum LoadError {
	AllocError(AllocError),
	WriteError(TryWriteError)
}

impl From<AllocError> for LoadError {
	fn from(e: AllocError) -> LoadError {
		LoadError::AllocError(e)
	}
}

trait AbstractQuery {
	fn process(&mut self, loader: &mut Loader, commands: &mut AutoCommandBufferBuilder) -> Result<(), LoadError>;
}

struct WriteQuery<T> {
	raw_buffer: Arc<RawBuffer>,
	pending_state: Option<Arc<PendingState>>,
	data: Option<T>
}

impl<T: 'static + Content + Send + Sync> AbstractQuery for WriteQuery<T> {
	fn process(&mut self, loader: &mut Loader, commands: &mut AutoCommandBufferBuilder) -> Result<(), LoadError> {
		unsafe {
			loader.write(commands, self.raw_buffer.clone(), self.data.take().unwrap(), self.pending_state.take())
		}
	}
}

enum Query {
	/// Generic query, used to call the `write` method with different types.
	Any(Box<dyn AbstractQuery>),

	/// Memory allocation.
	Alloc {
		raw_buffer: Arc<RawBuffer>,
		mem_reqs: MemoryRequirements,
		host_cached: bool
	}
}

/// Handle to a loader queue.
///
/// Every method of this type sends a query to the loader, which will eventually process the
/// query.
/// None of these methods are blocking.
pub struct LoaderQueue {
	device: Arc<Device>,
	queue: Arc<AtomicQueue<Query>>
}

impl LoaderQueue {
	pub fn device(&self) -> &Arc<Device> {
		&self.device
	}

	/// Allocate memory and bind it to the given buffer.
	pub unsafe fn alloc(&self, raw_buffer: Arc<RawBuffer>, mem_reqs: MemoryRequirements, host_cached: bool) {
		self.queue.push(Query::Alloc {
			raw_buffer, mem_reqs, host_cached
		});
	}

	/// Write some data to the given buffer and notyfy ths given pending state (if any) when it is done.
	pub unsafe fn write<T: 'static + Content + Sync + Send>(&self, raw_buffer: Arc<RawBuffer>, data: T, pending_state: Option<Arc<PendingState>>) {
		self.queue.push(Query::Any(Box::new(WriteQuery {
			raw_buffer,
			pending_state,
			data: Some(data)
		})));
	}
}

/// Memory allocation and transfer manager.
pub struct Loader {
	transfer_queue: Arc<Queue>,

	queue: Arc<AtomicQueue<Query>>,
	memory_pool: Arc<StdMemoryPool>,

	pending_states: Vec<Arc<PendingState>>
}

impl Loader {
	/// Create a new loader with a handle to its queue.
	pub fn new(transfer_queue: Arc<Queue>) -> (Loader, LoaderQueue) {
		let device = transfer_queue.device().clone();
		let queue = Arc::new(AtomicQueue::new());
		let loader = Loader {
			transfer_queue,
			queue: queue.clone(),
			memory_pool: StdMemoryPool::new(device.clone()),
			pending_states: Vec::new()
		};
		(loader, LoaderQueue { device, queue })
	}

	/// Loader device.
	pub fn device(&self) -> &Arc<Device> {
		self.transfer_queue.device()
	}

	/// Process all pending queries.
	///
	/// This will actually block the thread until all the queries has been processed.
	pub fn flush(&mut self) -> Result<(), LoadError> {
		if let Ok(query) = self.queue.pop() {
			let mut commands = AutoCommandBufferBuilder::new(self.device().clone(), self.transfer_queue.family()).unwrap();

			self.process_query(query, &mut commands)?;
			while let Ok(query) = self.queue.pop() {
				self.process_query(query, &mut commands)?
			}

			let future = commands.build().unwrap().execute(self.transfer_queue.clone()).unwrap();
			future.then_signal_fence_and_flush().unwrap().wait(None);

			for state in self.pending_states.drain(..) {
				state.set_ready()
			}
		}

		Ok(())
	}

	/// Process a single query.
	///
	/// Each query can add commands to the input command buffer, and add some pending states
	/// to be notified when the query is processed.
	fn process_query(&mut self, query: Query, commands: &mut AutoCommandBufferBuilder) -> Result<(), LoadError> {
		match query {
			Query::Any(mut query) => {
				query.process(self, commands)
			},
			Query::Alloc { raw_buffer, mem_reqs, host_cached } => {
				unsafe { self.alloc(raw_buffer, mem_reqs, host_cached)? };
				Ok(())
			}
		}
	}

	/// Allocate a new memory buffer.
	unsafe fn alloc(&mut self, raw_buffer: Arc<RawBuffer>, mem_reqs: MemoryRequirements, host_cached: bool) -> Result<(), AllocError> {
		let mem = self.memory_pool.alloc_from_requirements(
			&mem_reqs,
			AllocLayout::Linear,
			MappingRequirement::Map,
			DedicatedAlloc::Buffer(&raw_buffer.handle),
			|m| {
				if m.is_host_cached() == host_cached {
					AllocFromRequirementsFilter::Preferred
				} else {
					AllocFromRequirementsFilter::Allowed
				}
			},
		)?;

		debug_assert!((mem.offset() % mem_reqs.alignment) == 0);
		debug_assert!(mem.mapped_memory().is_some());

		raw_buffer.bind_memory(mem);

		Ok(())
	}

	/// Write data to a buffer.
	unsafe fn write<T: 'static + Content + Sync + Send>(&mut self, commands: &mut AutoCommandBufferBuilder, raw_buffer: Arc<RawBuffer>, data: T, pending_state: Option<Arc<PendingState>>) -> Result<(), LoadError> {
		{
			let mut guard = raw_buffer.try_write::<T>();
			match guard {
				Ok(mut buffer) => {
					std::ptr::write(buffer.deref_mut() as *mut T, data);
				},
				Err(TryWriteError::ForeignBuffer) => {
					let (raw_src_buffer, mem_reqs) = RawBuffer::unbound(self.device(), BufferUsage::transfer_source(), raw_buffer.len())?;
					self.alloc(raw_src_buffer.clone(), mem_reqs, true)?;
					{
						let mut guard = raw_src_buffer.try_write::<T>();
						match guard {
							Ok(mut src_buffer) => {
								std::ptr::write(src_buffer.deref_mut() as *mut T, data);
							},
							Err(TryWriteError::ForeignBuffer) => unreachable!(),
							Err(e) => return Err(LoadError::WriteError(e))
						}
					}

					commands.copy_buffer(
						RawBufferAccess::<T>::new(raw_src_buffer),
						RawBufferAccess::<T>::new(raw_buffer.clone())
					).unwrap();
				},
				Err(e) => return Err(LoadError::WriteError(e))
			}
		}

		if let Some(pending_state) = pending_state {
			self.pending_states.push(pending_state)
		}

		Ok(())
	}
}
