use std::{
	marker::PhantomData,
};

pub struct RefMap<T, F, U: ?Sized> where F: Fn(&T) -> &U {
	t: T,
	f: F,
	u: PhantomData<U>
}

impl<T, F, U: ?Sized> RefMap<T, F, U> where F: Fn(&T) -> &U {
	pub fn new(t: T, f: F) -> Self {
		Self {
			t,
			f,
			u: PhantomData
		}
	}
}

impl<T, F, U: ?Sized> std::ops::Deref for RefMap<T, F, U> where F: Fn(&T) -> &U {
	type Target = U;

	fn deref(&self) -> &U {
		(self.f)(&self.t)
	}
}

impl<T, F, U: ?Sized> AsRef<U> for RefMap<T, F, U> where F: Fn(&T) -> &U {
	fn as_ref(&self) -> &U {
		(self.f)(&self.t)
	}
}
