use std::{
	sync::Arc,
	ops::Deref,
	marker::PhantomData
};
use magma::{
	Device,
	framebuffer::RenderPass
};
use ::scene::{
	Scene,
	Map,
	Id,
	Ref,
};
use crate::View;

mod target;
mod pov;
mod generator;

pub use target::Target;
pub use generator::Generator;
pub use pov::PointOfView;

pub struct Worker<R: Target, T, E, P: PointOfView<T, E>, G: Generator<T>> {
	inner: Inner<R, T, E, G>,
	point_of_view: P
}

impl<R: Target, T, E, P: PointOfView<T, E>, G: Generator<T>> Worker<R, T, E, P, G> {
	pub fn new(render_target: R, point_of_view: P, generator: G) -> Self {
		Self {
			inner: Inner {
				render_target,
				generator,
				views: Map::new(),
				e: PhantomData
			},
			point_of_view
		}
	}
}

impl<R: Target, T, E, P: PointOfView<T, E>, G: Generator<T>> cycles::Worker<Scene<T, E>> for Worker<R, T, E, P, G> {
	fn cycle(&mut self, scene: &Scene<T, E>) {
		self.point_of_view.cycle(scene);
		for id in self.point_of_view.visible_objects() {
			let object = scene.get(id);
			self.inner.render_object(scene, object);
		}
	}

	fn apply(&mut self, _scene: &mut Scene<T, E>) {
		// nothing to apply in the scene.
	}
}

struct Inner<R: Target, T, E, G: Generator<T>> {
	render_target: R,
	generator: G,
	views: Map<T, View>,
	e: PhantomData<E>
}

impl<R: Target, T, E, G: Generator<T>> Inner<R, T, E, G> {
	fn render_object(&mut self, scene: &Scene<T, E>, object: Ref<T>) {
		let view = self.views.get(object.id());
		panic!("TODO render view")
	}
}