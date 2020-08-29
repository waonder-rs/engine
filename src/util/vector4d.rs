use super::Tensor2d;

#[repr(packed)]
pub struct Vector4D<T> {
	pub x: T,
	pub y: T,
	pub z: T,
	pub w: T
}

pub type Vertex4D<T> = Vector4D<T>;

unsafe impl<T> Tensor2d for Vector4D<T> {
	const DIM: (usize, usize) = (1, 4);
}

impl<T> Vector4D<T> {
	pub fn new(x: T, y: T, z: T, w: T) -> Vector4D<T> {
		Vector4D {
			x: x,
			y: y,
			z: z,
			w: w
		}
	}
}

impl<T: Default> Default for Vector4D<T> {
	fn default() -> Vector4D<T> {
		Vector4D {
			x: T::default(),
			y: T::default(),
			z: T::default(),
			w: T::default()
		}
	}
}
