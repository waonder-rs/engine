use std::sync::{
	Arc,
	Weak
};
use std::collections::HashSet;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use crate::{
	util::{
		Matrix4x4,
		Vector3d
	},
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

	center: Vector3d<f32>,
	distance: f32,
	position: (f32, f32), // surface coordinates (polar, azimuthal).

	/// Perspective projection
	projection: Matrix4x4<f32>
}

impl Satellite {
	pub fn new(projection: Matrix4x4<f32>, center: Vector3d<f32>, distance: f32, position: (f32, f32)) -> Satellite {
		let world_position = Vector3d::new(distance, 0.0, 0.0);

		let mut transformation = Matrix4x4::translation(center);
		transformation *= Matrix4x4::rotation(position.0, Vector3d::new(0.0, 0.0, 1.0));
		transformation *= Matrix4x4::rotation(position.1, Vector3d::new(0.0, 1.0, 0.0));
		transformation *= Matrix4x4::translation(-world_position);
		transformation *= Matrix4x4::looking_at(-world_position, Vector3d::new(0.0, 0.0, 1.0)).inverted().unwrap();

		Satellite {
			transformation,
			parent: None,
			children: HashSet::new(),

			center,
			distance,
			position,

			projection
		}
	}

	pub fn move_by(&mut self, delta_polar: f32, delta_azimuthal: f32) {
		self.position = (self.position.0 + delta_polar, self.position.1 + delta_azimuthal);
		let world_position = Vector3d::new(self.distance, 0.0, 0.0);

		self.transformation = Matrix4x4::translation(self.center);
		self.transformation *= Matrix4x4::rotation(-self.position.0, Vector3d::new(0.0, 0.0, 1.0));
		self.transformation *= Matrix4x4::rotation(-self.position.1, Vector3d::new(0.0, 1.0, 0.0));
		self.transformation *= Matrix4x4::translation(-world_position);
		self.transformation *= Matrix4x4::looking_at(-world_position, Vector3d::new(0.0, 0.0, 1.0)).inverted().unwrap();
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
