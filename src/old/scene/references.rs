use std::sync::{
	Arc,
	Weak
};
use std::hash::{
	Hash,
	Hasher
};
use std::cell::UnsafeCell;
use std::ops::{
	CoerceUnsized,
};
use std::marker::Unsize;
use super::Scene;

pub struct Ref<T: ?Sized>(pub(crate) *const u8, pub(crate) Arc<UnsafeCell<T>>);

impl<T: ?Sized> Ref<T> {
	pub fn new(scene: &Scene, value: T) -> Ref<T> where T: Sized {
		Ref(scene.token.as_ref() as *const u8, Arc::new(UnsafeCell::new(value)))
	}

	pub fn downgrade(r: &Ref<T>) -> WeakRef<T> {
		WeakRef(r.0, Arc::downgrade(&r.1))
	}
}

impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<Ref<U>> for Ref<T> {}

unsafe impl<T: ?Sized + Send + Sync> Send for Ref<T> {}
unsafe impl<T: ?Sized + Send + Sync> Sync for Ref<T> {}

impl<T: ?Sized> Clone for Ref<T> {
	fn clone(&self) -> Ref<T> {
		Ref(self.0, self.1.clone())
	}
}

impl<T: ?Sized> Hash for Ref<T> {
	fn hash<H: Hasher>(&self, h: &mut H) {
		Arc::as_ptr(&self.1).hash(h)
	}
}

impl<T: ?Sized> PartialEq for Ref<T> {
	fn eq(&self, other: &Ref<T>) -> bool {
		Arc::ptr_eq(&self.1, &other.1)
	}
}

impl<T: ?Sized> Eq for Ref<T> {}

pub struct WeakRef<T: ?Sized>(pub(crate) *const u8, pub(crate) Weak<UnsafeCell<T>>);

impl<T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<WeakRef<U>> for WeakRef<T> {}

unsafe impl<T: ?Sized + Send + Sync> Send for WeakRef<T> {}
unsafe impl<T: ?Sized + Send + Sync> Sync for WeakRef<T> {}

impl<T: ?Sized> WeakRef<T> {
	#[inline]
	pub fn upgrade(&self) -> Option<Ref<T>> {
		match self.1.upgrade() {
			Some(arc) => Some(Ref(self.0, arc)),
			None => None
		}
	}
}

impl<T: ?Sized> Clone for WeakRef<T> {
	fn clone(&self) -> WeakRef<T> {
		WeakRef(self.0, self.1.clone())
	}
}
