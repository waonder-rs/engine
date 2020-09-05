use std::sync::{
	Arc,
	Weak
};
use std::collections::HashSet;
use parking_lot::{
	Mutex,
	MutexGuard,
	MappedMutexGuard
};
use vulkano::{
	pipeline::{
		GraphicsPipeline,
		GraphicsPipelineAbstract,
		depth_stencil::DepthStencil
	},
	framebuffer::Subpass,
	command_buffer::AutoCommandBufferBuilder,
	sync::GpuFuture
};
use crate::{
	util::{
		Matrix4x4,
		Vector3d
	},
	Geometry,
	geometry,
	Material,
	RenderTarget
};
use super::{
	Scene,
	Node,
	NodeRef,
	WeakNodeRef,
};

pub struct Object {
	transformation: Matrix4x4<f32>,
	parent: Option<WeakNodeRef>,
	children: HashSet<NodeRef>,

	geometry: Arc<dyn Geometry>,
	projection: Arc<dyn geometry::Projection>,
	material: Arc<dyn Material>,

	pipeline: Mutex<Option<Arc<dyn GraphicsPipelineAbstract + Send + Sync>>>
}

impl Object {
	/// Create a new object.
	pub fn new(parent: Option<NodeRef>, geometry: Arc<dyn Geometry>, projection: Arc<dyn geometry::Projection>, material: Arc<dyn Material>) -> Object {
		let parent = match parent {
			Some(parent) => Some(NodeRef::downgrade(&parent)),
			None => None
		};

		Object {
			transformation: Matrix4x4::identity(),
			parent,
			children: HashSet::new(),
			geometry, projection, material,
			pipeline: Mutex::new(None)
		}
	}

	/// Build a graphics pipeline for this object.
	///
	/// TODO share graphics pipelines.
	fn rebuild_pipeline(&self, target: &dyn RenderTarget, pipeline: &mut MutexGuard<Option<Arc<dyn GraphicsPipelineAbstract + Send + Sync>>>) {
		let mut builder = GraphicsPipeline::start();
		let builder = builder.render_pass(Subpass::from(target.render_pass().clone(), 0).unwrap());

		// Vertex input
		let builder = builder.vertex_input_single_buffer::<geometry::Vertex>(); // TODO
		let builder = builder.vertex_shader(self.projection.shader().entry_point(), ());

		// Input assembly
		let builder = builder.triangle_list();

		// Viewport
		let builder = builder.viewports_dynamic_scissors_irrelevant(1);

		// Rasterization
		let builder = builder.fragment_shader(self.material.shader().entry_point(), ());

		// Depth test
		let builder = builder.depth_stencil(DepthStencil::simple_depth_test());

		// Build.
		pipeline.replace(Arc::new(builder.build(target.device().clone()).unwrap()));
	}

	/// Drawing pipeline for the given render target.
	pub fn pipeline<'a>(&'a self, target: &RenderTarget) -> MappedMutexGuard<'a, Arc<dyn GraphicsPipelineAbstract + Send + Sync>> {
		let mut guard = self.pipeline.lock();

		if guard.is_some() {
			//let pipeline = guard.as_ref().unwrap();
			return MutexGuard::map(guard, |p| p.as_mut().unwrap())
		}

		self.rebuild_pipeline(target, &mut guard);

		MutexGuard::map(guard, |p| p.as_mut().unwrap())
	}
}

impl Node for Object {
	fn transformation(&self) -> &Matrix4x4<f32> {
		&self.transformation
	}

	fn transformation_mut(&mut self) -> &mut Matrix4x4<f32> {
		&mut self.transformation
	}

	fn parent(&self) -> Option<&WeakNodeRef> {
		self.parent.as_ref()
	}

	fn parent_mut(&mut self) -> &mut Option<WeakNodeRef> {
		&mut self.parent
	}

	fn children(&self) -> &HashSet<NodeRef> {
		&self.children
	}

	fn children_mut(&mut self) -> &mut HashSet<NodeRef> {
		&mut self.children
	}

	fn draw(&self, _scene: &Scene, target: &RenderTarget, builder: &mut AutoCommandBufferBuilder, projection: &Matrix4x4<f32>) {
		let projection = projection * self.transformation();
		// println!("projection:\n{}", projection);
		builder.draw_indexed(self.pipeline(target).clone(), target.dynamic_state(), vec![self.geometry.vertex_buffer().clone()], self.geometry.index_buffer().clone(), (), projection).unwrap();
	}
}
