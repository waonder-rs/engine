use crate::{
	util::Matrix4x4,
	scene::{
		Node,
		Scene,
		View
	}
};

mod satellite;

pub use satellite::Satellite;

pub trait Camera: Node {
	fn projection(&self) -> &Matrix4x4<f32>;

	fn world_projection(&self, scene: &Scene) -> Matrix4x4<f32> {
		self.projection() * self.world_transformation(scene).inverted().unwrap()
	}
}

impl<'s, 'n> View<'s, 'n, dyn Camera> {
	#[inline]
	pub fn world_projection(&self) -> Matrix4x4<f32> {
		self.value.world_projection(self.scene)
	}
}
