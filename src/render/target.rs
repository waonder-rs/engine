use std::{
	sync::Arc,
	ops::Deref
};
use magma::{
	Device,
	framebuffer::RenderPass
};

pub trait Target {
	fn device(&self) -> &Arc<Device>;

	fn render_pass(&self) -> &Arc<RenderPass>;
}

impl<T: Deref> Target for T where T::Target: Target {
	fn device(&self) -> &Arc<Device> {
		Deref::deref(self).device()
	}

	fn render_pass(&self) -> &Arc<RenderPass> {
		Deref::deref(self).render_pass()
	}
}