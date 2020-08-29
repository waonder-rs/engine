use std::sync::Arc;
use vulkano::{
	device::Device,
	pipeline::shader::GraphicsShaderType,
	descriptor::{
		pipeline_layout::PipelineLayoutDescPcRange,
		descriptor::ShaderStages
	}
};
use crate::{
	util::Matrix4x4,
	shader,
	Shader
};

mod standard;

pub use standard::Standard;

pub trait Projection: Sync + Send {
	fn shader(&self) -> &VertexShader;
}

pub struct VertexShader(Shader);

impl VertexShader {
	pub unsafe fn new(device: &Arc<Device>, vspir: &[u8], input: shader::Input, output: shader::Output) -> VertexShader {
		VertexShader(Shader::new(device, vspir, GraphicsShaderType::Vertex, input, output))
	}
}

impl std::ops::Deref for VertexShader {
	type Target = Shader;

	fn deref(&self) -> &Shader {
		&self.0
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

/// Push-constant used to store the projection matrices.
const PROJECTION_PC_DESCRIPTOR: PipelineLayoutDescPcRange = PipelineLayoutDescPcRange {
	offset: 0,
	size: 128,
	stages: ShaderStages {
		vertex: true,
		tessellation_control: false,
		tessellation_evaluation: false,
		geometry: false,
		fragment: false,
		compute: false
	}
};

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
