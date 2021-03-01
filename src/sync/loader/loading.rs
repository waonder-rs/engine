use std::{
	sync::Arc,
	mem::ManuallyDrop
};
use once_cell::sync::OnceCell;

/// Loading or loaded value.
pub struct Loading<T>(Arc<OnceCell<Arc<T>>>);

impl<T> Loading<T> {
	pub fn new() -> (Self, Handle<T>) where T: 'static {
		let inner = Arc::new(OnceCell::new());
		let this = Self(inner.clone());
		let handle = Handle(Box::new(move |value| {
			inner.set(Arc::new(value)).ok().expect("unable to set loaded value");
		}));
		(this, handle)
	}

	pub fn mapped<U, F>(f: F) -> (Self, Handle<U>) where T: 'static, F: 'static + FnOnce(U) -> T {
		let inner = Arc::new(OnceCell::new());
		let this = Self(inner.clone());
		let handle = Handle(Box::new(move |value| {
			inner.set(Arc::new(f(value))).ok().expect("unable to set loaded value");
		}));
		(this, handle)
	}

	pub fn get(&self) -> Option<&Arc<T>> {
		self.0.get()
	}
}

pub struct Handle<T>(Box<dyn FnOnce(T) -> ()>);

impl<T> Handle<T> {
	pub fn prepare(self, value: T) -> Prepared<T> {
		Prepared {
			f: ManuallyDrop::new(self.0),
			value: ManuallyDrop::new(value)
		}
	}
}

pub struct Prepared<T> {
	f: ManuallyDrop<Box<dyn FnOnce(T) -> ()>>,
	value: ManuallyDrop<T>
}

impl<T> Drop for Prepared<T> {
	fn drop(&mut self) {
		let value = unsafe { ManuallyDrop::take(&mut self.value) };
		let f = unsafe { ManuallyDrop::take(&mut self.f) };
		f(value)
	}
}

impl<T> std::ops::Deref for Prepared<T> {
	type Target = T;

	fn deref(&self) -> &T {
		&self.value
	}
}