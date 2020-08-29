use std::sync::Arc;
use vulkano::{
	device::Device,
	format::Format
};
use crate::shader::{
	Input,
	Output
};
use super::{
	Projection,
	VertexShader
};

/// Standard projection.
pub struct Standard {
	shader: VertexShader
}

impl Standard {
	pub fn new(device: &Arc<Device>) -> Standard {
		let vspir = include_bytes!("shaders/standard.spv");

		let shader = unsafe {
			let mut input = Input::empty();
			input.add("position", 0..1, Format::R32G32B32Sfloat);
			input.add_push_constant(super::PROJECTION_PC_DESCRIPTOR);

			VertexShader::new(device, vspir, input, Output::empty())
		};

		Standard {
			shader
		}
	}
}

impl Projection for Standard {
	fn shader(&self) -> &VertexShader {
		&self.shader
	}
}
