use std::collections::HashSet;
use crate::{
	util::Matrix4x4,
	// RenderTarget
};
use super::{
	Scene,
	Ref,
	WeakRef,
	View
};

/// Node.
///
/// It is assumed that the node graph is acyclic.
pub trait Node: Send + Sync {
	fn transformation(&self) -> &Matrix4x4<f32>;

	fn transformation_mut(&mut self) -> &mut Matrix4x4<f32>;

	fn parent(&self) -> Option<&WeakNodeRef>;

	/// Users should never use this method.
	fn parent_mut(&mut self) -> &mut Option<WeakNodeRef>;

	fn children(&self) -> &HashSet<NodeRef>;

	/// Users should never use this method.
	fn children_mut(&mut self) -> &mut HashSet<NodeRef>;

	/// Compute the world transformation of this node.
	fn world_transformation(&self, scene: &Scene) -> Matrix4x4<f32> {
		match self.parent() {
			Some(weak_parent) => match weak_parent.upgrade() {
				Some(parent) => {
					scene.get(&parent).world_transformation() * self.transformation()
				},
				None => *self.transformation()
			},
			None => *self.transformation()
		}
	}

	// fn draw(&self, scene: &Scene, target: &RenderTarget, builder: &mut AutoCommandBufferBuilder, projection: &Matrix4x4<f32>) {
	// 	let projection = projection * self.transformation();
	// 	for child in self.children() {
	// 		scene.get(child).draw(target, builder, &projection);
	// 	}
	// }
}

pub type NodeRef = Ref<dyn Node>;
pub type WeakNodeRef = WeakRef<dyn Node>;

impl<'s, 'n> View<'s, 'n, dyn Node> {
	/// Compute the world transformation of this node.
	#[inline]
	pub fn world_transformation(&self) -> Matrix4x4<f32> {
		self.value.world_transformation(self.scene)
	}

	// #[inline]
	// pub fn draw(&self, target: &RenderTarget, builder: &mut AutoCommandBufferBuilder, projection: &Matrix4x4<f32>) {
	// 	self.value.draw(self.scene, target, builder, projection)
	// }
}

impl<T: 'static + Node> From<Ref<T>> for NodeRef {
	fn from(node: Ref<T>) -> NodeRef {
		node as NodeRef
	}
}

impl<'a, T: 'static + Node> From<&'a Ref<T>> for NodeRef {
	fn from(node: &'a Ref<T>) -> NodeRef {
		node.clone() as NodeRef
	}
}
