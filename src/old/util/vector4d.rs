use std::fmt;
use super::Tensor2d;

#[repr(packed)]
pub struct Vector4d<T> {
	pub x: T,
	pub y: T,
	pub z: T,
	pub w: T
}

pub type Vertex4D<T> = Vector4d<T>;

unsafe impl<T> Tensor2d for Vector4d<T> {
	const DIM: (usize, usize) = (1, 4);
}

impl<T> Vector4d<T> {
	pub fn new(x: T, y: T, z: T, w: T) -> Vector4d<T> {
		Vector4d {
			x: x,
			y: y,
			z: z,
			w: w
		}
	}
}

impl<T: Default> Default for Vector4d<T> {
	fn default() -> Vector4d<T> {
		Vector4d {
			x: T::default(),
			y: T::default(),
			z: T::default(),
			w: T::default()
		}
	}
}

impl<T: Copy + fmt::Display> fmt::Display for Vector4d<T> {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "({}, {}, {}, {})", self.x, self.y, self.z, self.w)
	}
}
