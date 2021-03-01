use std::{
	mem::ManuallyDrop,
	sync::Arc
};
use crossbeam_queue::SegQueue;
use magma::{
	Device,
	DeviceOwned,
	sync::fence
};

pub struct FencePool {
	device: Arc<Device>,
	inner: Arc<Inner>
}

impl FencePool {
	pub fn new(device: &Arc<Device>) -> Self {
		Self {
			device: device.clone(),
			inner: Arc::new(Inner {
				queue: SegQueue::new()
			})
		}
	}

	pub fn get(&self) -> Result<Fence, fence::CreationError> {
		let raw_fence = match self.inner.queue.pop() {
			Some(raw_fence) => raw_fence,
			None => fence::Raw::new(&self.device)?
		};

		Ok(Fence {
			pool: self.inner.clone(),
			handle: ManuallyDrop::new(raw_fence)
		})
	}
}

struct Inner {
	queue: SegQueue<fence::Raw>
}

pub struct Fence {
	pool: Arc<Inner>,
	handle: ManuallyDrop<fence::Raw>
}

impl DeviceOwned for Fence {
	fn device(&self) -> &Arc<Device> {
		self.handle.device()
	}
}

impl magma::sync::Fence for Fence {
	fn handle(&self) -> &fence::VulkanFence {
		self.handle.handle()
	}
}

impl Drop for Fence {
	fn drop(&mut self) {
		let handle = unsafe { ManuallyDrop::take(&mut self.handle) };
		self.pool.queue.push(handle)
	}
}