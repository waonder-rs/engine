use std::sync::Arc;
use glam::{
	Vec3,
	Mat4
};
use magma::{
	Format,
	pipeline,
	command,
	mem,
	device
};
use parking_lot::{
	Mutex,
	MutexGuard,
	MappedMutexGuard
};
use crate::{
	render,
	sync::Loader
};
use super::{
	geometry,
	Geometry,
	Material
};

pub struct Object {
	/// Geometry.
	geometry: Geometry,

	/// Geometry projection.
	projection: Arc<dyn geometry::Projection>,

	/// Material.
	material: Arc<dyn Material>,
	
	/// Graphics pipeline.
	pipeline: Mutex<Option<Arc<pipeline::Graphics>>>
}

impl Object {
	pub fn draw<C: render::Context, B: command::Buffer>(
		&self,
		context: &C,
		commands: &mut command::buffer::Recorder<B>,
		projection: &Mat4
	) {
		if let Some(vertex_buffer) = self.geometry.vertex_buffer(context.loader(), context.graphics_queue().into()) {
			if let Some(index_buffer) = self.geometry.index_buffer(0, context.loader(), context.graphics_queue().into()) {
				let mut vertex_buffers = mem::Buffers::new();
				vertex_buffers.push(vertex_buffer.clone());

				commands.bind_graphics_pipeline(&self.pipeline(context.target()));
				commands.bind_vertex_buffers(0, vertex_buffers, &[0]);
				commands.bind_index_buffer(index_buffer.clone(), 0);
				commands.draw_indexed(index_buffer.len() as u32, 1, 0, 0, 0);
			}
		}
	}

	/// Build a graphics pipeline for this object.
	///
	/// TODO share graphics pipelines.
	fn rebuild_pipeline<T: render::Target>(
		&self,
		target: &T,
		pipeline_guard: &mut MutexGuard<Option<Arc<pipeline::Graphics>>>
	) {
		use pipeline::{
			InputAssembly,
			input_assembly,
			Viewport,
			Scissor,
			ColorBlend,
			color_blend::{
				self,
				BlendFactor
			},
			DynamicState
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
			std::mem::size_of::<Vec3>() as u32,
			pipeline::vertex_input::Rate::Vertex
		));
		vertex_input.add_attribute(pipeline::vertex_input::Attribute::new(
			0, // location
			0, // binding
			Format::R32G32Sfloat,
			0 // offset
		));

		// let set_layouts = &[
		// 	pipeline::layout::Set::new(target.device(), &[]).expect("unable to create set")
		// ];
		let set_layouts = &[]; // TODO

		let push_constant_ranges = &[]; // TODO

		let layout = Arc::new(pipeline::Layout::new(
			target.device(),
			set_layouts,
			push_constant_ranges
		).expect("unable to create layout"));

		let pipeline = pipeline::Graphics::new(
			target.device(),
			&stages,
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
	pub fn pipeline<'a, T: render::Target>(&'a self, target: &T) -> MappedMutexGuard<'a, Arc<pipeline::Graphics>> {
		let mut guard = self.pipeline.lock();

		if guard.is_some() {
			//let pipeline = guard.as_ref().unwrap();
			return MutexGuard::map(guard, |p| p.as_mut().unwrap())
		}

		self.rebuild_pipeline(target, &mut guard);

		MutexGuard::map(guard, |p| p.as_mut().unwrap())
	}
}