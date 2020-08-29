use std::sync::Arc;
use std::cell::UnsafeCell;
use std::collections::HashSet;
use vulkano::command_buffer::AutoCommandBufferBuilder;
use crate::{
	util::Matrix4x4,
	RenderTarget
};

mod references;
mod view;
mod node;
mod object;
pub mod camera;

pub use references::*;
pub use view::*;
pub use node::*;
pub use object::Object;
pub use camera::Camera;

pub struct Scene {
	token: Box<u8>, // to uniquely identify the scene
	roots: HashSet<NodeRef>
}

impl Scene {
	pub fn new() -> Scene {
		Scene {
			token: Box::new(0),
			roots: HashSet::new()
		}
	}

	pub fn add_root<N: Into<NodeRef>>(&mut self, node: N) -> bool {
		self.roots.insert(node.into())
	}

	pub fn new_node<T: 'static + Node>(&self, node: T) -> Ref<T> {
		Ref::new(self, node)
	}

	/// Shortut to create a new root node.
	///
	/// Equivalent to
	/// ```
	/// let node = scene.new_node(value);
	/// scene.add_root(node);
	/// node
	/// ```
	pub fn new_root<T: 'static + Node>(&mut self, node: T) -> Ref<T> {
		let node_ref = self.new_node(node);
		self.add_root(node_ref.clone());
		node_ref
	}

	pub fn owns<T: ?Sized>(&self, id: &Ref<T>) -> bool {
		self.token.as_ref() as *const u8 == id.0
	}

	pub fn get<'s, 'n, T: ?Sized>(&'s self, id: &'n Ref<T>) -> View<'s, 'n, T> {
		if self.owns(id) {
			View {
				scene: self,
				value: unsafe { &*id.1.get() }
			}
		} else {
			panic!("wrong scene")
		}
	}

	pub fn get_mut<'s, 'n, T: ?Sized>(&'s mut self, id: &'n Ref<T>) -> ViewMut<'s, 'n, T> {
		if self.owns(id) {
			ViewMut {
				scene: self,
				value: unsafe { &mut *id.1.get() }
			}
		} else {
			panic!("wrong scene")
		}
	}

	pub fn draw(&self, target: &RenderTarget, builder: &mut AutoCommandBufferBuilder, transformation: &Matrix4x4<f32>) {
		for node in &self.roots {
			self.get(node).draw(target, builder, transformation);
		}
	}
}
