use std::marker::PhantomData;
use std::sync::{
	Arc,
	atomic::{
		Ordering,
		AtomicUsize
	}
};
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

use crate::mem::AllocError;

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

enum DeviceLock {
	Reading(AtomicUsize),
	Writing(usize)
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
			lock: RwLock::new(DeviceLock::Reading(AtomicUsize::new(0)))
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
				match &*lock {
					DeviceLock::Reading(ref count) if count.load(Ordering::SeqCst) == 0 => {
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
					_ => {
						Err(TryWriteError::DeviceLocked)
					}
				}
			},
			None => Err(TryWriteError::HostLocked)
		}
	}
}

unsafe impl DeviceOwned for RawBuffer {
	fn device(&self) -> &Arc<Device> {
		&self.handle.device()
	}
}

unsafe impl BufferAccess for RawBuffer {
	#[inline]
	fn inner(&self) -> BufferInner {
		BufferInner {
			buffer: &self.handle,
			offset: 0
		}
	}

	#[inline]
	fn size(&self) -> usize {
		self.len()
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
		(self.handle.key(), 0)
	}

	#[inline]
	fn try_gpu_lock(&self, exclusive_access: bool, _: &Queue) -> Result<(), AccessError> {
		if exclusive_access {
			let mut lock = match self.lock.try_write() {
				Some(lock) => lock,
				None => return Err(AccessError::AlreadyInUse)
			};

			match *lock {
				DeviceLock::Reading(ref count) if count.load(Ordering::SeqCst) == 0 => (),
				_ => return Err(AccessError::AlreadyInUse)
			}

			*lock = DeviceLock::Writing(1)
		} else {
			let mut lock = match self.lock.try_read() {
				Some(lock) => lock,
				None => return Err(AccessError::AlreadyInUse)
			};

			match *lock {
				DeviceLock::Reading(ref count) => count.fetch_add(1, Ordering::SeqCst),
				DeviceLock::Writing(_) => return Err(AccessError::AlreadyInUse)
			};
		}

		Ok(())
	}

	#[inline]
	unsafe fn increase_gpu_lock(&self) {
		{
			let lock = self.lock.read();
			if let DeviceLock::Reading(ref count) = *lock {
				count.fetch_add(1, Ordering::SeqCst);
				return
			}
		}

		{
			let mut lock = self.lock.write();
			if let DeviceLock::Writing(ref mut count) = *lock {
				*count += 1
			} else {
				unreachable!()
			}
		}
	}

	#[inline]
	unsafe fn unlock(&self) {
		{
			let lock = self.lock.read();
			if let DeviceLock::Reading(ref count) = *lock {
				count.fetch_sub(1, Ordering::SeqCst);
				return
			}
		}

		{
			let mut lock = self.lock.write();
			if let DeviceLock::Writing(ref mut count) = *lock {
				if *count != 1 {
					*count -= 1
				}
			} else {
				panic!("buffer unlocked too many times")
			}

			// back to reading.
			*lock = DeviceLock::Reading(AtomicUsize::new(0))
		}
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
		self.buffer.inner()
	}

	#[inline]
	fn size(&self) -> usize {
		self.buffer.size()
	}

	#[inline]
	fn conflicts_buffer(&self, other: &dyn BufferAccess) -> bool {
		self.buffer.conflicts_buffer(other)
	}

	#[inline]
	fn conflicts_image(&self, other: &dyn ImageAccess) -> bool {
		self.buffer.conflicts_image(other)
	}

	#[inline]
	fn conflict_key(&self) -> (u64, usize) {
		self.buffer.conflict_key()
	}

	#[inline]
	fn try_gpu_lock(&self, exclusive_access: bool, queue: &Queue) -> Result<(), AccessError> {
		self.buffer.try_gpu_lock(exclusive_access, queue)
	}

	#[inline]
	unsafe fn increase_gpu_lock(&self) {
		self.buffer.increase_gpu_lock()
	}

	#[inline]
	unsafe fn unlock(&self) {
		self.buffer.unlock()
	}
}

unsafe impl<T: ?Sized + Content> TypedBufferAccess for RawBufferAccess<T> {
	type Content = T;
}
