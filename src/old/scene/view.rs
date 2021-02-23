use std::ops::{
	Deref,
	DerefMut,
	CoerceUnsized,
	// DispatchFromDyn
};
use std::marker::Unsize;
use super::Scene;

pub struct View<'s, 't, T: ?Sized> {
	pub(crate) scene: &'s Scene,
	pub(crate) value: &'t T
}

impl<'s, 't, T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<View<'s, 't, U>> for View<'s, 't, T> {}
// impl<'s, 't, T: ?Sized + Unsize<U>, U: ?Sized> DispatchFromDyn<View<'s, 't, U>> for View<'s, 't, T> {}

impl<'s, 't, T: ?Sized> Deref for View<'s, 't, T> {
	type Target = T;

	#[inline]
	fn deref(&self) -> &T {
		self.value
	}
}

pub struct ViewMut<'s, 't, T: ?Sized> {
	pub(crate) scene: &'s Scene,
	pub(crate) value: &'t mut T
}

impl<'s, 't, T: ?Sized + Unsize<U>, U: ?Sized> CoerceUnsized<ViewMut<'s, 't, U>> for ViewMut<'s, 't, T> {}

impl<'s, 't, T: ?Sized> Deref for ViewMut<'s, 't, T> {
	type Target = T;

	#[inline]
	fn deref(&self) -> &T {
		self.value
	}
}

impl<'s, 't, T: ?Sized> DerefMut for ViewMut<'s, 't, T> {
	#[inline]
	fn deref_mut(&mut self) -> &mut T {
		self.value
	}
}
