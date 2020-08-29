use crate::util::Matrix4x4;
use super::Projection;

/// Perspective projection.
pub struct Perspective {
	matrix: Matrix4x4<f32>
}

impl Perspective {
	pub fn new(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Perspective {
		Perspective {
			matrix: Matrix4x4::perspective(left, right, bottom, top, near, far)
		}
	}

	/// Create a new orthographic projection with the given horizontal field of view.
	/// The aspect ratio is the ratio of x (width) to y (height).
	pub fn fovx(fovx: f32, aspect: f32, near: f32, far: f32) -> Perspective {
		let right = near * (fovx * 0.5).tan();
		let top = right / aspect;

		Perspective {
			matrix: Matrix4x4::perspective(-right, right, -top, top, near, far)
		}
	}
}

impl Projection for Perspective {
	fn matrix(&self) -> &Matrix4x4<f32> {
		&self.matrix
	}
}
