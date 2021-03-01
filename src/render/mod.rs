use std::{
	sync::Arc,
	ops::Deref,
	marker::PhantomData
};
use magma::{
	device,
	Device,
	framebuffer::RenderPass
};
use ::scene::{
	Scene,
	Map,
	Id,
	Ref,
};
use crate::{
	View,
	sync::Loader
};

mod target;
mod context;
mod pov;
mod generator;

pub use target::Target;
pub use context::Context;
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
				context: WorkerContext {
					target: render_target,
					graphics_queue: panic!("TODO"),
					loader: panic!("TODO")
				},
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
	context: WorkerContext<R>,
	generator: G,
	views: Map<T, View>,
	e: PhantomData<E>
}

impl<R: Target, T, E, G: Generator<T>> Inner<R, T, E, G> {
	fn render_object(&mut self, scene: &Scene<T, E>, object: Ref<T>) {
		let view = self.views.get(object.id());
		
		let view = match view {
			Some(view) => view,
			None => {
				let view = self.generator.view(&object);
				self.views.set(object.id(), view);
				self.views.get(object.id()).unwrap()
			}
		};

		panic!("TODO")
		// view.draw(&self.context, commands, projection)
	}
}

struct WorkerContext<R: Target> {
	target: R,
	graphics_queue: device::Queue,
	loader: Loader
}

impl<R: Target> Context for WorkerContext<R> {
	type Target = R;

	fn target(&self) -> &R {
		&self.target
	}

	fn graphics_queue(&self) -> &device::Queue {
		&self.graphics_queue
	}

	fn loader(&self) -> &Loader {
		&self.loader
	}
}