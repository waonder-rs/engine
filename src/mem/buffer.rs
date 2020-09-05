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
	PendingState
};

pub enum TryWriteError {
	/// The buffer is not accessible to the host.
	ForeignBuffer,

	/// The buffer is used by the host.
	HostLocked,

	/// The buffer is used by the GPU device.
	DeviceLocked,

	/// The buffer is not bound to any memory.
	///
	/// This can only occur with `RawBuffer`.
	Unbound
}

#[derive(Clone, Copy)]
enum DeviceLock {
	Free,
	Locked
}

pub struct WriteGuard<'a, T: ?Sized> {
	inner: CpuAccess<'a, T>,
	lock: RwLockWriteGuard<'a, DeviceLock>
}

impl<'a, T: ?Sized> Deref for WriteGuard<'a, T> {
	type Target = T;

	fn deref(&self) -> &T {
		self.inner.deref()
	}
}

impl<'a, T: ?Sized> DerefMut for WriteGuard<'a, T> {
	fn deref_mut(&mut self) -> &mut T {
		self.inner.deref_mut()
	}
}

pub struct RawBufferAccess<T: ?Sized> {
	buffer: Arc<RawBuffer>,
	t: PhantomData<T>
}

impl<T: ?Sized> RawBufferAccess<T> {
	pub unsafe fn new(buffer: Arc<RawBuffer>) -> RawBufferAccess<T> {
		RawBufferAccess {
			buffer,
			t: PhantomData
		}
	}
}

unsafe impl<T: ?Sized> DeviceOwned for RawBufferAccess<T> {
	fn device(&self) -> &Arc<Device> {
		self.buffer.device()
	}
}

unsafe impl<T: ?Sized> BufferAccess for RawBufferAccess<T> {
	#[inline]
	fn inner(&self) -> BufferInner {
		BufferInner {
			buffer: &self.buffer.handle,
			offset: 0
		}
	}

	#[inline]
	fn size(&self) -> usize {
		self.buffer.len()
	}

	#[inline]
	fn conflicts_buffer(&self, other: &dyn BufferAccess) -> bool {
		self.conflict_key() == other.conflict_key()
	}

	#[inline]
	fn conflicts_image(&self, other: &dyn ImageAccess) -> bool {
		false
	}

	#[inline]
	fn conflict_key(&self) -> (u64, usize) {
		(self.buffer.handle.key(), 0)
	}

	#[inline]
	fn try_gpu_lock(&self, exclusive_access: bool, _: &Queue) -> Result<(), AccessError> {
		if exclusive_access {
			return Err(AccessError::ExclusiveDenied)
		}

		panic!("TODO")
	}

	#[inline]
	unsafe fn increase_gpu_lock(&self) {
		panic!("TODO")
	}

	#[inline]
	unsafe fn unlock(&self) {
		panic!("TODO")
	}
}

unsafe impl<T: ?Sized + Content> TypedBufferAccess for RawBufferAccess<T> {
	type Content = T;
}

/// Raw buffer without type information.
///
/// May not be bound to any memory yet.
pub struct RawBuffer {
	/// Underlying vulkano buffer object.
	pub(crate) handle: UnsafeBuffer,

	/// Bound memory, if any.
	memory: OnceCell<Box<dyn MemoryPoolAlloc + Sync + Send>>,

	/// Buffer access lock.
	lock: RwLock<DeviceLock>
}

impl RawBuffer {
	/// Create an unbound raw buffer.
	pub unsafe fn unbound(device: &Arc<Device>, usage: BufferUsage, size: usize) -> Result<(Arc<RawBuffer>, MemoryRequirements), AllocError> {
		let sharing: Sharing<core::iter::Empty<_>> = Sharing::Exclusive;

		let (handle, mem_reqs) = match UnsafeBuffer::new(device.clone(), size, usage, sharing, SparseLevel::none()) {
			Ok(b) => b,
			Err(BufferCreationError::AllocError(e)) => return Err(e),
			Err(_) => unreachable!() // We don't use sparse binding.
		};

		Ok((Arc::new(RawBuffer {
			handle,
			memory: OnceCell::new(),
			lock: RwLock::new(DeviceLock::Free)
		}), mem_reqs))
	}

	fn device(&self) -> &Arc<Device> {
		self.handle.device()
	}

	/// Bind some memory to the buffer.
	///
	/// Does nothing if the buffer is already bound.
	pub unsafe fn bind_memory<M: 'static + MemoryPoolAlloc + Sync + Send>(&self, memory: M) {
		if let Ok(_) = self.memory.set(Box::new(memory)) {
			let memory = self.memory.get().unwrap();
			self.handle.bind_memory(memory.memory(), memory.offset());
		}
	}

	#[inline]
	pub fn len(&self) -> usize {
		self.handle.size()
	}

	pub unsafe fn try_write<'a, T: ?Sized>(&'a self) -> Result<WriteGuard<'a, T>, TryWriteError> where T: Content {
		match self.lock.try_write() {
			Some(lock) => {
				match *lock {
					DeviceLock::Free => {
						match self.memory.get() {
							Some(memory) => {
								let offset = memory.offset();
								let range = offset..offset + self.handle.size();

								match memory.mapped_memory() {
									Some(mapped_mem) => {
										Ok(WriteGuard {
											inner: mapped_mem.read_write(range),
											lock
										})
									},
									None => Err(TryWriteError::ForeignBuffer)
								}
							},
							None => Err(TryWriteError::Unbound)
						}
					},
					DeviceLock::Locked => {
						Err(TryWriteError::DeviceLocked)
					}
				}
			},
			None => Err(TryWriteError::HostLocked)
		}
	}
}

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
}
