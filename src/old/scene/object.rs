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
use magma::{
	Format,
	pipeline::{
		self,
		DynamicState
	}
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

	pipeline: Mutex<Option<Arc<pipeline::Graphics>>>
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
	fn rebuild_pipeline(&self, target: &dyn RenderTarget, pipeline_guard: &mut MutexGuard<Option<Arc<pipeline::Graphics>>>) {
		use pipeline::{
			InputAssembly,
			input_assembly,
			Viewport,
			Scissor,
			ColorBlend,
			color_blend::{
				self,
				BlendFactor
			}
		};

		let stages = unsafe {
			pipeline::stage::Vertex::new(
				self.projection.shader().entry_point(),
				pipeline::stage::Fragment::new(
					self.material.shader().entry_point()
				)
			)
		};

		let mut vertex_input = pipeline::VertexInput::new();
		vertex_input.add_binding(pipeline::vertex_input::Binding::new(
			0,
			std::mem::size_of::<Vector3d>(),
			pipeline::vertex_input::Rate::Vertex
		));
		vertex_input.add_attribute(pipeline::vertex_input::Attribute::new(
			0, // location
			0, // binding
			Format::R32G32SFloat,
			0 // offset
		));

		let set_layouts = &[
			pipeline::layout::Set::new(target.device(), &[]).expect("unable to create set")
		];

		let push_constant_ranges = &[];

		let layout = pipeline::Layout::new(
			target.device(),
			set_layouts,
			push_constant_ranges
		).expect("unable to create layout");

		let pipeline = pipeline::Graphics::new(
			target.device(),
			stages,
			vertex_input,
			InputAssembly::new(input_assembly::Topology::TriangleList, false),
			None, // no tesselation
			[Viewport::default()], [Scissor::default()],
			pipeline::Rasterization::new(
				false,
				false,
				pipeline::rasterization::PolygonMode::Fill,
				pipeline::rasterization::CullMode::Back,
				pipeline::rasterization::FrontFace::Clockwise,
				None,
				1.0
			),
			pipeline::Multisample::default(), // no multisampling
			None,
			None,
			ColorBlend::new(None, [0.0, 0.0, 0.0, 0.0]).with_attachment(color_blend::Attachment::new(
				Some(color_blend::AttachmentBlend::new(
					BlendFactor::SourceAlpha,
					BlendFactor::OneMinusSourceAlpha,
					color_blend::Operation::Add,
					BlendFactor::One,
					BlendFactor::Zero,
					color_blend::Operation::Add
				)),
				color_blend::ColorComponents::rgba()
			)),
			&layout,
			target.render_pass().subpass(0).unwrap(),
			(DynamicState::Viewport, DynamicState::Scissor)
		).expect("unable to build pipeline");

		// Build.
		pipeline_guard.replace(Arc::new(pipeline));
	}

	/// Drawing pipeline for the given render target.
	pub fn pipeline<'a>(&'a self, target: &dyn RenderTarget) -> MappedMutexGuard<'a, Arc<pipeline::Graphics>> {
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

	// fn draw(&self, _scene: &Scene, target: &RenderTarget, builder: &mut AutoCommandBufferBuilder, projection: &Matrix4x4<f32>) {
	// 	let projection = projection * self.transformation();
	// 	// println!("projection:\n{}", projection);
	// 	builder.draw_indexed(self.pipeline(target).clone(), target.dynamic_state(), vec![self.geometry.vertex_buffer().clone()], self.geometry.index_buffer().clone(), (), projection).unwrap();
	// }
}
