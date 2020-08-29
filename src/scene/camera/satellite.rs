use std::sync::{
	Arc,
	Weak
};
use std::collections::HashSet;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use crate::{
	util::Matrix4x4,
	RenderTarget,
	Scene,
	Node,
	NodeRef,
	WeakNodeRef
};
use super::Camera;

pub struct Satellite {
	transformation: Matrix4x4<f32>,
	parent: Option<WeakNodeRef>,
	children: HashSet<NodeRef>,

	/// Perspective projection
	projection: Matrix4x4<f32>
}

impl Satellite {
	pub fn new(projection: Matrix4x4<f32>) -> Satellite {
		Satellite {
			transformation: Matrix4x4::identity(),
			parent: None,
			children: HashSet::new(),

			projection
		}
	}
}

impl Node for Satellite {
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
}

impl Camera for Satellite {
	fn projection(&self) -> &Matrix4x4<f32> {
		&self.projection
	}
}
