use std::{
	rc::Rc,
	sync::Arc
};
use magma::{
	mem::{
		buffer
	},
	sync::SharingQueues
};
use crate::util::RefMap;
use crate::sync::{
	Loader,
	loader::Loading,
};
use once_cell::unsync::OnceCell;

pub mod projection;
pub use projection::Projection;

pub struct Geometry {
	source: Rc<geometer::AbstractGeometry>,
	vertex_buffer: OnceCell<Loading<buffer::Bound>>,
	index_buffers: Vec<OnceCell<Loading<buffer::Typed<u32>>>>,
}

impl Geometry {
	pub fn new(source: geometer::AbstractGeometry) -> Self {
		let index_buffer_count = source.precisions().len();
		let mut index_buffers = Vec::new();
		index_buffers.resize_with(index_buffer_count, || OnceCell::new());

		Self {
			source: Rc::new(source),
			vertex_buffer: OnceCell::new(),
			index_buffers
		}
	}

	pub fn vertex_buffer(&self, loader: &Loader, sharing_queues: SharingQueues) -> Option<&Arc<buffer::Bound>> {
		self.vertex_buffer.get_or_init(move || {
			let vertices: RefMap<_, _, [u8]> = RefMap::new(self.source.clone(), |s| s.vertices());
			loader.load_untyped(vertices, buffer::Usage::VertexBuffer, sharing_queues)
		}).get()
	}

	pub fn index_buffer(&self, precision: usize, loader: &Loader, sharing_queues: SharingQueues) -> Option<&Arc<buffer::Typed<u32>>> {
		let precision = std::cmp::min(precision, self.source.precisions().len());

		let index_buffer = &self.index_buffers[precision];
		index_buffer.get_or_init(move || {
			let indices: RefMap<_, _, [u32]> = RefMap::new(self.source.clone(), move |s| s.precisions()[precision].indices());
			loader.load(indices, buffer::Usage::IndexBuffer, sharing_queues)
		}).get()
	}
}