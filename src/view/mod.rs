use glam::Mat4;
use magma::command;
use crate::render;

pub mod geometry;
pub mod material;
pub mod object;

pub use geometry::Geometry;
pub use material::Material;
pub use object::Object;

/// Object graphical representation.
pub enum View {
	Object(Object)
}

impl View {
	pub fn draw<C: render::Context, B: command::Buffer>(&self, context: &C, commands: &mut command::buffer::Recorder<B>, projection: &Mat4) {
		match self {
			View::Object(obj) => obj.draw(context, commands, projection)
		}
	}
}