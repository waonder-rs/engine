use crate::util::Matrix4x4;

mod orthographic;
mod perspective;

pub use orthographic::Orthographic;
pub use perspective::Perspective;

pub trait Projection {
	/// The projection matrix.
	fn matrix(&self) -> &Matrix4x4<f32>;
}
