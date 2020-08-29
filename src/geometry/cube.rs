use std::sync::Arc;
use vulkano::{
	device::Queue,
	buffer::{
		BufferAccess,
		BufferUsage,
		TypedBufferAccess,
		immutable::ImmutableBuffer
	},
	sync::GpuFuture
};

use crate::{
	loader,
	Loader
};
use super::{
	Geometry,
	Vertex
};

/// Simple cube geometry.
pub struct Cube {
	vertex_buffer: Arc<dyn BufferAccess + Sync + Send>,
	index_buffer: Arc<dyn TypedBufferAccess<Content = [u32]> + Sync + Send>
}

impl Cube {
	/// Create a cube geometry on the GPU side.
	///
	/// Returns a handle to the cube geometry and a GPU future that tracks the geometry data transfert status.
	/// The geometry is unusable for a render until the future is completed.
	pub fn new(size: f32, loader: &Loader) -> loader::Future<Cube> {
		//        y
		//        |
		//        2 ----- 6
		//      / |     / |
		//    3 ----- 7   |
		//    |   0 --|-- 4 -- x
		//    | /     | /
		//    1 ----- 5
		//   /
		// z

		let vertices = [
			Vertex::new(-size, -size, -size), // 0
			Vertex::new(-size, -size, size), // 1
			Vertex::new(-size, size, -size), // 2
			Vertex::new(-size, size, size), // 3
			Vertex::new(size, -size, -size), // 4
			Vertex::new(size, -size, size), // 5
			Vertex::new(size, size, -size), // 6
			Vertex::new(size, size, size) // 7
		];

		let indexes: Vec<u32> = vec![
			0, 4, 6, 0, 6, 2,
			0, 2, 1, 2, 3, 1,
			0, 1, 5, 5, 4, 0,
			2, 6, 7, 7, 3, 2,
			7, 6, 4, 4, 5, 7,
			3, 7, 5, 5, 1, 3
		];

		let (vertex_buffer, vertex_future) = ImmutableBuffer::from_data(vertices, BufferUsage::vertex_buffer(), loader.queue().clone()).unwrap();
		let (index_buffer, index_future) = ImmutableBuffer::from_iter(indexes.into_iter(), BufferUsage::index_buffer(), loader.queue().clone()).unwrap();

		loader.load(vertex_future.join(index_future), Cube {
			vertex_buffer: vertex_buffer as Arc<dyn BufferAccess + Sync + Send>,
			index_buffer: index_buffer as Arc<dyn TypedBufferAccess<Content = [u32]> + Sync + Send>
		})
	}
}

impl Geometry for Cube {
	fn vertex_buffer(&self) -> &Arc<dyn BufferAccess + Sync + Send> {
		&self.vertex_buffer
	}

	fn index_buffer(&self) -> &Arc<dyn TypedBufferAccess<Content = [u32]> + Sync + Send> {
		&self.index_buffer
	}
}
