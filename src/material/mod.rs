use std::sync::Arc;
use vulkano::{
	device::Device,
	pipeline::shader::GraphicsShaderType
};
use crate::{
	shader,
	Shader
};

mod depth;
pub use depth::Depth;

pub trait Material: Sync + Send {
	fn shader(&self) -> &FragmentShader;
}

pub struct FragmentShader(Shader);

impl FragmentShader {
	pub unsafe fn new(device: &Arc<Device>, vspir: &[u8], input: shader::Input, output: shader::Output) -> FragmentShader {
		FragmentShader(Shader::new(device, vspir, GraphicsShaderType::Fragment, input, output))
	}
}

impl std::ops::Deref for FragmentShader {
	type Target = Shader;

	fn deref(&self) -> &Shader {
		&self.0
	}
}
