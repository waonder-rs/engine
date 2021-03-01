use std::{
	mem::ManuallyDrop,
	sync::Arc,
	rc::Rc
};
use crossbeam_queue::SegQueue;
use magma::{
	device,
	Device,
	DeviceOwned,
	command
};

pub struct CommandBufferPool {
	pool: Rc<command::Pool>,
	inner: Arc<Inner>
}

impl CommandBufferPool {
	pub fn new(transfert_queue: &device::Queue) -> Result<Self, command::pool::CreationError> {
		let pool = command::Pool::new(transfert_queue.device(), transfert_queue.family())?;

		Ok(Self {
			pool: Rc::new(pool),
			inner: Arc::new(Inner {
				queue: SegQueue::new()
			})
		})
	}

	pub fn get(&self) -> Result<Buffer, command::pool::AllocError> {
		let raw_fence = match self.inner.queue.pop() {
			Some(raw_fence) => raw_fence,
			None => self.pool.allocate(1)?.into_iter().next().unwrap()
		};

		Ok(Buffer {
			pool: self.inner.clone(),
			handle: ManuallyDrop::new(raw_fence)
		})
	}
}

struct Inner {
	queue: SegQueue<command::buffer::Raw>
}

pub struct Buffer {
	pool: Arc<Inner>,
	handle: ManuallyDrop<command::buffer::Raw>
}

impl DeviceOwned for Buffer {
	fn device(&self) -> &Arc<Device> {
		self.handle.device()
	}
}

impl command::Buffer for Buffer {
	fn handle(&self) -> command::buffer::VulkanBuffer {
		self.handle.handle()
	}
}

impl Drop for Buffer {
	fn drop(&mut self) {
		let handle = unsafe { ManuallyDrop::take(&mut self.handle) };
		self.pool.queue.push(handle)
	}
}