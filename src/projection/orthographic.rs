use crate::util::Matrix4x4;
use super::Projection;

/// Orthographic projection.
pub struct Orthographic {
	matrix: Matrix4x4<f32>
}

impl Orthographic {
	/// Create a new orthographic projection.
	pub fn new(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Orthographic {
		Orthographic {
			matrix: Matrix4x4::orthographic(left, right, bottom, top, near, far)
		}
	}
}

impl Projection for Orthographic {
	fn matrix(&self) -> &Matrix4x4<f32> {
		&self.matrix
	}
}
