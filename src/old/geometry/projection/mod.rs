use std::sync::Arc;
use magma::pipeline::shader;
use crate::{
	util::Matrix4x4,
	// shader,
	// Shader
};

mod standard;

pub use standard::Standard;

pub trait Projection: Sync + Send {
	fn shader(&self) -> &VertexShader;
}

pub struct VertexShader(Arc<shader::Module>);

impl VertexShader {
	// pub unsafe fn new(device: &Arc<Device>, vspir: &[u8], input: shader::Input, output: shader::Output) -> VertexShader {
	// 	VertexShader(Shader::new(device, vspir, GraphicsShaderType::Vertex, input, output))
	// }

	pub fn entry_point(&self) -> shader::EntryPoint {
		unsafe {
			self.0.entry_point("main")
		}
	}
}

pub struct CameraProjection {
	pub modelview: Matrix4x4<f32>,
	pub proj: Matrix4x4<f32>
}

impl Default for CameraProjection {
	fn default() -> CameraProjection {
		CameraProjection {
			modelview: Matrix4x4::identity(),
			proj: Matrix4x4::identity()
		}
	}
}

// /// Push-constant used to store the projection matrices.
// const PROJECTION_PC_DESCRIPTOR: PipelineLayoutDescPcRange = PipelineLayoutDescPcRange {
// 	offset: 0,
// 	size: 128,
// 	stages: ShaderStages {
// 		vertex: true,
// 		tessellation_control: false,
// 		tessellation_evaluation: false,
// 		geometry: false,
// 		fragment: false,
// 		compute: false
// 	}
// };

// const PROJECTION_BINDING_DESCRIPTOR: DescriptorDesc = DescriptorDesc {
// 	ty: DescriptorDesc::Buffer(DescriptorBufferDesc {
// 		dynamic: None,
// 		storage: false
// 	}),
// 	array_count: 1,
// 	stages: ShaderStages {
// 		vertex: true,
// 		tessellation_control: false,
// 		tessellation_evaluation: false,
// 		geometry: false,
// 		fragment: false,
// 		compute: false
// 	},
// 	readonly: true
// };
