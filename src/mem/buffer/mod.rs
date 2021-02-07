use std::marker::PhantomData;
use std::future::Future;
use std::sync::Arc;
use std::ops::{
	Deref,
	DerefMut
};
use once_cell::sync::OnceCell;
use parking_lot::{
	RwLock,
	RwLockWriteGuard
};
use vulkano::{
	device::{
		Device,
		DeviceOwned,
		Queue
	},
	memory::{
		MemoryRequirements,
		Content,
		CpuAccess,
		pool::MemoryPoolAlloc
	},
	buffer::{
		sys::{
			UnsafeBuffer,
			BufferCreationError,
			SparseLevel
		},
		BufferInner,
		BufferUsage,
		BufferAccess,
		TypedBufferAccess
	},
	image::ImageAccess,
	sync::{
		Sharing,
		AccessError
	}
};

use super::{
	LoaderQueue,
	AllocError,
	Pending,
};

mod raw;

pub use raw::*;

pub struct Buffer<T: ?Sized> {
	raw: Arc<RawBuffer>,
	t: PhantomData<T>
}

impl<T: ?Sized> Buffer<T> {
	/// Allocate memory for a buffer through the given loader and fill it with the given data.
	///
	/// If the buffer's memory is not host accessible, a temporary host buffer is created and
	/// a transfert command is queued into the given loader to load the data from the host buffer
	/// into the remote buffer.
	pub fn from_data(&self, loader: &LoaderQueue, usage: BufferUsage, data: T) -> Result<Pending<Buffer<T>>, AllocError> where T: 'static + Sized + Content + Sync + Send {
		unsafe {
			// Create a new unbound buffer.
			let (raw_buffer, mem_reqs) = RawBuffer::unbound(loader.device(), usage, std::mem::size_of::<T>())?;

			let (loading_buffer, pending_state) = Pending::new(Buffer {
				raw: raw_buffer.clone(),
				t: PhantomData
			});

			loader.alloc(raw_buffer.clone(), mem_reqs, false);
			loader.write(raw_buffer, data, Some(pending_state));

			Ok(loading_buffer)
		}
	}

	pub fn try_write<'a>(&'a self) -> Result<WriteGuard<'a, T>, TryWriteError> where T: Content {
		unsafe {
			self.raw.try_write()
		}
	}

	pub fn access(&self) -> RawBufferAccess<T> {
		unsafe {
			RawBufferAccess::new(self.raw.clone())
		}
	}
}

unsafe impl<T: ?Sized> DeviceOwned for Buffer<T> {
	fn device(&self) -> &Arc<Device> {
		self.raw.device()
	}
}

unsafe impl<T: ?Sized> BufferAccess for Buffer<T> {
	#[inline]
	fn inner(&self) -> BufferInner {
		self.raw.inner()
	}

	#[inline]
	fn size(&self) -> usize {
		self.raw.size()
	}

	#[inline]
	fn conflicts_buffer(&self, other: &dyn BufferAccess) -> bool {
		self.raw.conflicts_buffer(other)
	}

	#[inline]
	fn conflicts_image(&self, other: &dyn ImageAccess) -> bool {
		self.raw.conflicts_image(other)
	}

	#[inline]
	fn conflict_key(&self) -> (u64, usize) {
		self.raw.conflict_key()
	}

	#[inline]
	fn try_gpu_lock(&self, exclusive_access: bool, queue: &Queue) -> Result<(), AccessError> {
		self.raw.try_gpu_lock(exclusive_access, queue)
	}

	#[inline]
	unsafe fn increase_gpu_lock(&self) {
		self.raw.increase_gpu_lock()
	}

	#[inline]
	unsafe fn unlock(&self) {
		self.raw.unlock()
	}
}

unsafe impl<T: ?Sized + Content> TypedBufferAccess for Buffer<T> {
	type Content = T;
}
