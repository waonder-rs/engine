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
	Material,
	FragmentShader
};

pub struct Depth {
	shader: FragmentShader
}

impl Depth {
	pub fn new(device: &Arc<Device>) -> Depth {
		let vspir = include_bytes!("shaders/depth.spv");

		let shader = unsafe {
			let mut output = Output::empty();
			output.add("color", 0..1, Format::R32G32B32Sfloat);

			FragmentShader::new(device, vspir, Input::empty(), output)
		};

		Depth {
			shader
		}
	}
}

impl Material for Depth {
	fn shader(&self) -> &FragmentShader {
		&self.shader
	}
}
