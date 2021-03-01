use std::sync::Arc;
use magma::pipeline::shader;
// use crate::{
// 	shader,
// 	Shader
// };

mod depth;
pub use depth::Depth;

pub trait Material: Sync + Send {
	fn shader(&self) -> &FragmentShader;
}

pub struct FragmentShader(Arc<shader::Module>);

impl FragmentShader {
	// pub unsafe fn new(device: &Arc<Device>, vspir: &[u8], input: shader::Input, output: shader::Output) -> FragmentShader {
	// 	FragmentShader(Shader::new(device, vspir, GraphicsShaderType::Fragment, input, output))
	// }

	pub fn entry_point(&self) -> shader::EntryPoint {
		unsafe {
			self.0.entry_point("main")
		}
	}
}