use std::sync::Arc;
// use vulkano::{
// 	buffer::{
// 		BufferAccess,
// 		TypedBufferAccess
// 	},
// 	pipeline::vertex::{
// 		Vertex as VulkanoVertex,
// 		VertexMemberInfo,
// 		VertexMemberTy
// 	}
// };

pub mod projection;
mod cube;

pub use projection::Projection;
pub use cube::Cube;

pub trait Geometry {
	fn vertex_buffer(&self) -> Arc<dyn Buffer>;

	fn index_buffer(&self) -> Arc<dyn Buffer>;

	// /// GPU accessible buffer to the geometry vertices.
	// fn vertex_buffer(&self) -> &Arc<dyn BufferAccess + Sync + Send>;

	// fn index_buffer(&self) -> &Arc<dyn TypedBufferAccess<Content = [u32]> + Sync + Send>;
}

pub struct Vertex {
	position: [f32; 3],
	// normal: [f32; 3]
}

impl Vertex {
	pub fn new(x: f32, y: f32, z: f32) -> Vertex {
		Vertex {
			position: [x, y, z]
		}
	}
}