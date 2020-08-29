mod algebra;
mod vector2d;
mod vector3d;
mod vector4d;
mod matrix4x4;

pub use algebra::*;
pub use vector2d::*;
pub use vector3d::*;
pub use vector4d::*;
pub use matrix4x4::*;

/// 2-dimentional tensors.
pub unsafe trait Tensor2d {
	/// Row-major tensor size.
	/// number of row first, the number of columns.
	const DIM: (usize, usize);
}
