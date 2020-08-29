extern crate vulkano;
extern crate vulkano_win;
extern crate winit;

use std::path::Path;
use std::fs::File;
use std::io::Read;
use std::sync::Arc;
use vulkano::{
	instance::{
		Instance,
		PhysicalDevice,
		QueueFamily
	},
	device::{
		Device,
		DeviceExtensions,
		QueuesIter
	},
	swapchain::{
		Swapchain,
		Surface,
		Capabilities,
		ColorSpace,
		CompositeAlpha,
		PresentMode,
		FullscreenExclusive
	},
	format::{
		Format,
		ClearValue
	},
	image::{
		ImageUsage,
		ImageLayout
	},
	pipeline::{
		shader::{
			ShaderModule,
			GraphicsShaderType,
			ShaderInterfaceDef,
			ShaderInterfaceDefEntry
		},
		GraphicsPipeline
	},
	framebuffer::{
		RenderPass,
		RenderPassDesc,
		RenderPassDescClearValues,
		PassDescription,
		PassDependencyDescription,
		AttachmentDescription,
		LoadOp,
		StoreOp,
		Subpass,
		Framebuffer
	},
	command_buffer::AutoCommandBufferBuilder,
	descriptor::{
		descriptor::DescriptorDesc,
		pipeline_layout::{
			PipelineLayoutDesc,
			PipelineLayoutDescPcRange
		}
	},
	sync::GpuFuture
};

use vulkano_win::VkSurfaceBuild;

use winit::{
	event_loop::EventLoop,
	window::{
		Window,
		WindowBuilder
	}
};

fn get_queue_family<'a>(physical_device: &'a PhysicalDevice, surface: &Surface<Window>) -> QueueFamily<'a> {
	// TODO we may want one queue for graphics, and another one for presentation.
	physical_device.queue_families().find(|&queue| {
		queue.supports_graphics() && surface.is_supported(queue).unwrap_or(false)
	}).unwrap()
}

fn get_device<'a>(physical_device: &'a PhysicalDevice, queue_family: QueueFamily<'a>) -> (Arc<Device>, QueuesIter) {
	// TODO check that this extension is supported?
	let device_ext = DeviceExtensions {
		khr_swapchain: true,
		..DeviceExtensions::none()
	};

	Device::new(
		physical_device.clone(),
		physical_device.supported_features(), // enabled features (all of them?)
		&device_ext,
		[(queue_family, 1.0)].iter().cloned()
	).unwrap()
}

// Choose a surface format and color space.
fn choose_format(surface_capabilities: &Capabilities) -> Option<(Format, ColorSpace)> {
	for (format, color_space) in &surface_capabilities.supported_formats {
		if *format == Format::B8G8R8A8Srgb && *color_space == ColorSpace::SrgbNonLinear {
			return Some((*format, *color_space))
		}
	}

	None
}

/// Load a shader module.
///
/// # Safety
/// The SPIR-V code is not validated or may require features that are not enabled.
unsafe fn load_shader_module<P: AsRef<Path>>(device: &Arc<Device>, path: P) -> Arc<ShaderModule> {
	let mut file = File::open(path).expect("Unable to open shader file");
	let mut buffer = Vec::new();
	file.read_to_end(&mut buffer).expect("Unable to read shader file");
	ShaderModule::new(device.clone(), &buffer).expect("Unable to load shader module")
}

struct CustomRenderPass {
	attachment: AttachmentDescription,
	draw_pass: PassDescription
}

impl CustomRenderPass {
	fn new<W>(swapchain: &Swapchain<W>) -> CustomRenderPass {
		CustomRenderPass {
			attachment: AttachmentDescription {
				format: swapchain.format(),
				samples: 1,
				load: LoadOp::Clear,
				store: StoreOp::Store,
				stencil_load: LoadOp::DontCare,
				stencil_store: StoreOp::DontCare,
				initial_layout: ImageLayout::Undefined,
				final_layout: ImageLayout::PresentSrc
			},
			draw_pass: PassDescription {
				color_attachments: vec![(0, ImageLayout::ColorAttachmentOptimal)],
				depth_stencil: None,
				input_attachments: vec![],
				resolve_attachments: vec![],
				preserve_attachments: vec![]
			}
		}
	}
}

unsafe impl RenderPassDescClearValues<Vec<ClearValue>> for CustomRenderPass {
	fn convert_clear_values(&self, values: Vec<ClearValue>) -> Box<dyn Iterator<Item = ClearValue>> {
		// FIXME: safety checks?
		Box::new(values.into_iter())
	}
}

unsafe impl RenderPassDesc for CustomRenderPass {
	fn num_attachments(&self) -> usize {
		1
	}

	fn attachment_desc(&self, num: usize) -> Option<AttachmentDescription> {
		if num == 0 {
			Some(self.attachment.clone())
		} else {
			None
		}
	}

	fn num_subpasses(&self) -> usize {
		1
	}

	fn subpass_desc(&self, num: usize) -> Option<PassDescription> {
		if num == 0 {
			Some(self.draw_pass.clone())
		} else {
			None
		}
	}

	fn num_dependencies(&self) -> usize {
		0
	}

	fn dependency_desc(&self, _: usize) -> Option<PassDependencyDescription> {
		None
	}
}

pub struct ShaderInterface {
	entries: Vec<ShaderInterfaceDefEntry>
}

unsafe impl ShaderInterfaceDef for ShaderInterface {
	type Iter = std::vec::IntoIter<ShaderInterfaceDefEntry>;

	fn elements(&self) -> Self::Iter {
		self.entries.clone().into_iter()
	}
}

pub struct EmptyShaderInterface;

unsafe impl ShaderInterfaceDef for EmptyShaderInterface {
	type Iter = std::option::IntoIter<ShaderInterfaceDefEntry>;

	fn elements(&self) -> Self::Iter {
		None.into_iter()
	}
}


#[derive(Clone)]
pub struct Layout;

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
		0
	}

	fn push_constants_range(&self, num: usize) -> Option<PipelineLayoutDescPcRange> {
		None
	}
}

fn main() {
	// Required extensions to render on windows.
	let required_extensions = vulkano_win::required_extensions();

	// Create a vulkan instance.
	let instance = match Instance::new(None, &required_extensions, None) {
		Ok(i) => i,
		Err(err) => panic!("Couldn't build instance: {:?}", err)
	};

	// Get a physical device.
	let physical_device = PhysicalDevice::enumerate(&instance).next().unwrap();
	println!("Name: {}", physical_device.name());

	// Event loop and surface.
	let event_loop = EventLoop::new();
	let surface = WindowBuilder::new()
		.build_vk_surface(&event_loop, instance.clone())
		.unwrap();
	let dimensions: [u32; 2] = surface.window().inner_size().into();

	// Create logical device (and queues).
	let queue_family = get_queue_family(&physical_device, &surface);
	let (device, mut queues) = get_device(&physical_device, queue_family);
	let queue = queues.next().unwrap();

	// Sruface-Swapchain capabilities.
	let surface_capabilities = surface.capabilities(physical_device).unwrap();

	// Create the swapchain.
	let (format, color_space) = choose_format(&surface_capabilities).expect("No appropriate format found");
	let (swapchain, images) = Swapchain::new(
		device.clone(),
		surface.clone(),
		surface_capabilities.min_image_count,
		format,
		dimensions, // TODO check if the dimensions are supported by the swapchain.
		1,
		ImageUsage::color_attachment(),
		&queue,
		surface_capabilities.current_transform,
		CompositeAlpha::Opaque, // ignore alpha component.
		PresentMode::Fifo, // guaranteed to exist.
		FullscreenExclusive::Default,
		true,
		color_space
	).unwrap();

	// Load the shader modules.
	let vertex_shader = unsafe { load_shader_module(&device, "examples/vertex.spv") };
	let fragment_shader = unsafe { load_shader_module(&device, "examples/fragment.spv") };

	let vertex_shader_input = ShaderInterface {
		entries: vec![
			ShaderInterfaceDefEntry {
				location: 0..1,
				format: Format::R32Sfloat, // TODO what is this?
				name: None
			}
		]
	};

	let fragment_shader_output = ShaderInterface {
		entries: vec![
			ShaderInterfaceDefEntry {
				location: 0..1,
				format: Format::R32Sfloat, // TODO what is this?
				name: None
			}
		]
	};

	let vs_entry_point = unsafe {
		vertex_shader.graphics_entry_point(
			::std::ffi::CStr::from_bytes_with_nul(b"main\0").unwrap(),
			vertex_shader_input,
			EmptyShaderInterface,
			Layout,
			GraphicsShaderType::Vertex
		)
	};

	let fs_entry_point = unsafe {
		fragment_shader.graphics_entry_point(
			::std::ffi::CStr::from_bytes_with_nul(b"main\0").unwrap(),
			EmptyShaderInterface,
			fragment_shader_output,
			Layout,
			GraphicsShaderType::Fragment
		)
	};

	let render_pass = Arc::new(RenderPass::new(device.clone(), CustomRenderPass::new(&swapchain)).unwrap());

	// Create the pipeline.
	let pipeline = Arc::new(GraphicsPipeline::start()
		// .vertex_input_single_buffer() // set the vertex input to a vertex buffer
		.vertex_shader(vs_entry_point, ()) // set the vertex shader
		.triangle_list() // set the topology
		.viewports_dynamic_scissors_irrelevant(1)
		.polygon_mode_fill() // rasterizer
		.fragment_shader(fs_entry_point, ()) // set the fragment shader
		.render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
		.build(device.clone())
		.unwrap()
	);

	let framebuffers: Vec<_> = images.iter().map(|image| {
		let fb = Framebuffer::start(render_pass.clone())
			.add(image.clone()).unwrap()
			.build().unwrap();
		Arc::new(fb)
	}).collect();

	let (i, _suboptimal, acquire_future) = vulkano::swapchain::acquire_next_image(swapchain.clone(), None).unwrap();

	let clear_values = vec![[0.0, 0.0, 1.0, 1.0].into()];
	let mut builder = AutoCommandBufferBuilder::primary(device.clone(), queue_family).unwrap();
	builder.begin_render_pass(framebuffers[0].clone(), false, clear_values).unwrap()
		.end_render_pass().unwrap();
	let command_buffer = builder.build().unwrap();

	let now = vulkano::sync::now(device.clone());
	let future = now.join(acquire_future).then_execute(queue.clone(), command_buffer).unwrap()
	.then_swapchain_present(queue.clone(), swapchain.clone(), i)
	.then_signal_fence_and_flush();

	future.unwrap();

	// Loop.
	event_loop.run(move |event, _, _| {
		// nothing yet.
	});
}
