#![feature(arbitrary_self_types)]
#![feature(coerce_unsized)]
// #![feature(dispatch_from_dyn)]
#![feature(unsize)]
extern crate vulkano;

use std::sync::Arc;
use vulkano::{
	device::Device,
	framebuffer::RenderPassAbstract,
	command_buffer::DynamicState
};

pub mod util;
pub mod sync;
pub mod mem;
pub mod shader;
pub mod geometry;
pub mod projection;
pub mod material;
pub mod scene;
pub mod loader;

pub use shader::Shader;
pub use geometry::Geometry;
pub use projection::Projection;
pub use material::Material;
pub use scene::*;
pub use loader::Loader;

pub trait RenderTarget {
	fn device(&self) -> &Arc<Device>;

	fn render_pass(&self) -> &Arc<dyn RenderPassAbstract + Send + Sync>;

	fn dynamic_state(&self) -> &DynamicState;
}
