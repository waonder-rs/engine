use std::ops::Range;
use std::borrow::Cow;
use std::sync::Arc;
use vulkano::{
	device::Device,
	format::Format,
	pipeline::shader::{
		ShaderModule,
		ShaderInterfaceDef,
		ShaderInterfaceDefEntry,
		GraphicsShaderType
	},
	descriptor::{
		pipeline_layout::{
			PipelineLayoutDesc,
			PipelineLayoutDescPcRange
		},
		descriptor::DescriptorDesc
	}
};

#[derive(Clone)]
pub struct Input {
	interface: Interface,
	layout: Layout
	// sets: Vec<DescriptorDesc>
}

impl Input {
	pub fn empty() -> Input {
		Input {
			interface: Interface::empty(),
			layout: Layout::default()
		}
	}

	pub fn add_push_constant(&mut self, pc_range: PipelineLayoutDescPcRange) {
		self.layout.pc_ranges.push(pc_range)
	}
}

impl std::ops::Deref for Input {
	type Target = Interface;

	fn deref(&self) -> &Interface {
		&self.interface
	}
}

impl std::ops::DerefMut for Input {
	fn deref_mut(&mut self) -> &mut Interface {
		&mut self.interface
	}
}

#[derive(Clone)]
pub struct Interface {
	entries: Vec<ShaderInterfaceDefEntry>
}

impl Interface {
	pub fn empty() -> Interface {
		Interface {
			entries: Vec::new()
		}
	}

	pub fn add(&mut self, name: &str, location: Range<u32>, format: Format) {
		self.entries.push(ShaderInterfaceDefEntry {
			location,
			format,
			name: Some(Cow::Owned(name.to_string()))
		})
	}
}

unsafe impl ShaderInterfaceDef for Interface {
	type Iter = std::vec::IntoIter<ShaderInterfaceDefEntry>;

	fn elements(&self) -> Self::Iter {
		self.entries.clone().into_iter()
	}
}

pub type Output = Interface;

#[derive(Clone, Default)]
pub struct Layout {
	pc_ranges: Vec<PipelineLayoutDescPcRange>
}

unsafe impl PipelineLayoutDesc for Layout {
	fn num_sets(&self) -> usize {
		0
	}

	fn num_bindings_in_set(&self, set: usize) -> Option<usize> {
		None
	}

	fn descriptor(&self, set: usize, binding: usize) -> Option<DescriptorDesc> {
		None
	}

	fn num_push_constants_ranges(&self) -> usize {
		self.pc_ranges.len()
	}

	fn push_constants_range(&self, num: usize) -> Option<PipelineLayoutDescPcRange> {
		self.pc_ranges.get(num).cloned()
	}
}

pub struct Shader {
	module: Arc<ShaderModule>,
	shader_type: GraphicsShaderType,
	input: Input,
	output: Output
}

impl Shader {
	/// Create a new shader from V-SPIR.
	///
	/// # Safety
	/// - The `vspir` buffer must hold valid V-SPIR bytecode.
	/// - `input` and `output` must represent the actual interface of the shader.
	/// - The entry point name must be `main`.
	pub unsafe fn new(device: &Arc<Device>, vspir: &[u8], shader_type: GraphicsShaderType, input: Input, output: Output) -> Shader {
		let module = ShaderModule::new(device.clone(), vspir).expect("Unable to load shader module");

		Shader {
			module,
			shader_type,
			input,
			output
		}
	}

	/// Get the vertex shader entry point.
	pub fn entry_point<S>(&self) -> vulkano::pipeline::shader::GraphicsEntryPoint<S, Interface, Interface, Layout> {
		unsafe {
			let input = self.input.clone();
			let output = self.output.clone();

			self.module.graphics_entry_point(
				::std::ffi::CStr::from_bytes_with_nul(b"main\0").unwrap(),
				input.interface,
				self.output.clone(),
				input.layout,
				self.shader_type
			)
		}
	}
}
