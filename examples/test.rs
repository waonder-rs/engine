#![feature(arbitrary_self_types)]

extern crate vulkano;
extern crate vulkano_win;
extern crate winit;
extern crate engine;

use std::sync::Arc;
use parking_lot::Mutex;
use bottle::Remote;
use vulkano::{
	instance::{
		Instance,
		PhysicalDevice,
		QueueFamily
	},
	device::{
		Device,
		DeviceExtensions,
		Queue,
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
		ImageLayout,
		SwapchainImage
	},
	framebuffer::{
		RenderPass,
		RenderPassAbstract,
		RenderPassDesc,
		RenderPassDescClearValues,
		PassDescription,
		PassDependencyDescription,
		AttachmentDescription,
		LoadOp,
		StoreOp,
		Framebuffer,
		FramebufferAbstract
	},
	pipeline::viewport::Viewport,
	command_buffer::{
		AutoCommandBufferBuilder,
		DynamicState
	},
	sync::{
		GpuFuture,
		SharingMode
	}
};

use vulkano_win::VkSurfaceBuild;

use winit::{
	event_loop::EventLoop,
	event::Event,
	window::{
		Window as WinitWindow,
		WindowBuilder
	}
};

use engine::{
	util::{
		Matrix4x4,
		Vector3D
	},
	sync,
	loader::Loader,
	geometry,
	projection,
	material,
	camera,
	Camera,
	Object,
	Scene,
	RenderTarget,
	Node,
	Ref
};

fn get_queue_family<'a>(physical_device: &'a PhysicalDevice, surface: &Surface<WinitWindow>) -> QueueFamily<'a> {
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

struct CustomRenderPass {
	attachment: AttachmentDescription,
	draw_pass: PassDescription
}

impl CustomRenderPass {
	fn new(format: Format) -> CustomRenderPass {
		CustomRenderPass {
			attachment: AttachmentDescription {
				format,
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

/// A window with a swapchain.
pub struct Window {
	surface: Arc<Surface<WinitWindow>>,
	format: Format,
	color_space: ColorSpace,
	queues: Queues,
	target: WindowRenderTarget,
	swapchain: Option<WindowSwapchain>,
	future: Option<Box<dyn GpuFuture + Send + Sync>>,
	camera: Ref<dyn Camera>
}

pub struct WindowRenderTarget {
	device: Arc<Device>,
	render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
	dynamic_state: DynamicState,
}

pub struct WindowSwapchain {
	handle: Arc<Swapchain<WinitWindow>>,
	images: Vec<Arc<SwapchainImage<WinitWindow>>>,
	framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
	optimal: bool
}

pub struct Queues {
	graphics: Arc<Queue>,
	transfert: Arc<Queue>,
	presentation: Arc<Queue>,
}

impl Window {
	fn new<T>(event_loop: &EventLoop<T>, physical_device: &PhysicalDevice, camera: Ref<dyn Camera>) -> Window {
		let surface = WindowBuilder::new().build_vk_surface(&event_loop, physical_device.instance().clone()).unwrap();
		surface.window().set_resizable(true);

		// Create logical device (and queues).
		let queue_family = get_queue_family(&physical_device, &surface);
		let (device, mut queues) = get_device(&physical_device, queue_family);
		let queue = queues.next().unwrap();

		let surface_capabilities = surface.capabilities(device.physical_device()).unwrap();
		let (format, color_space) = choose_format(&surface_capabilities).expect("No appropriate format found");
		let render_pass = Arc::new(RenderPass::new(device.clone(), CustomRenderPass::new(format)).unwrap());

		Window {
			target: WindowRenderTarget {
				device: device.clone(),
				render_pass: render_pass as Arc<dyn RenderPassAbstract + Send + Sync>,
				dynamic_state: DynamicState::none(),
			},
			surface,
			format,
			color_space,
			queues: Queues {
				graphics: queue.clone(),
				transfert: queue.clone(), // graphics queues support transfert commands.
				presentation: queue
			},
			swapchain: None,
			future: None,
			camera
		}
	}

	fn dimensions(&self) -> [u32; 2] {
		self.surface.window().inner_size().into()
	}

	fn rebuild_swapchain(&mut self) {
		// println!("rebuilding the swapchain...");

		// Surface-Swapchain capabilities.
		let surface_capabilities = self.surface.capabilities(self.target.device.physical_device()).unwrap();

		let (handle, images) = match self.swapchain.take() {
			Some(old_swapchain) => {
				old_swapchain.handle.recreate_with_dimensions(self.surface.window().inner_size().into()).unwrap()
			},
			None => {
				Swapchain::new(
					self.target.device.clone(),
					self.surface.clone(),
					surface_capabilities.min_image_count,
					self.format,
					self.surface.window().inner_size().into(), // TODO check if the dimensions are supported by the swapchain.
					1,
					ImageUsage::color_attachment(),
					SharingMode::Exclusive, // TODO Image sharing mode
					surface_capabilities.current_transform,
					CompositeAlpha::Opaque, // ignore alpha component.
					PresentMode::Fifo, // guaranteed to exist.
					FullscreenExclusive::Default,
					true,
					self.color_space
				).unwrap()
			}
		};

		let dimensions = handle.dimensions();
		// println!("surface size: {:?}", self.surface.window().inner_size());
		// println!("swapchain images size: {:?}", dimensions);

		self.target.dynamic_state.viewports = Some(vec![Viewport {
			origin: [0.0, 0.0],
			dimensions: [dimensions[0] as f32, dimensions[1] as f32],
			depth_range: -1.0..1.0
		}]);

		let framebuffers: Vec<_> = images.iter().map(|image| {
			let fb = Framebuffer::start(self.target.render_pass.clone())
				.add(image.clone()).unwrap()
				.build().unwrap();
			Arc::new(fb) as Arc<dyn FramebufferAbstract + Send + Sync>
		}).collect();

		self.swapchain.replace(WindowSwapchain {
			handle, images, framebuffers,
			optimal: true
		});
	}

	fn swapchain_is_compatible(&self) -> bool {
		match self.swapchain.as_ref() {
			Some(swapchain) => {
				// if swapchain.handle.dimensions() != self.dimensions() {
				// 	println!("surface size: {:?}, incompatible with", self.surface.window().inner_size());
				// 	println!("swapchain images size: {:?}.", swapchain.handle.dimensions());
				// }
				//
				// if !swapchain.optimal {
				// 	println!("suboptimal swapchain.")
				// }

				swapchain.optimal && swapchain.handle.dimensions() == self.dimensions()
			},
			None => false
		}
	}

	fn render(&mut self, scene: &Scene) {
		if !self.swapchain_is_compatible() {
			self.rebuild_swapchain();
		}

		let mut swapchain = self.swapchain.as_mut().unwrap();

		// Acquire the next image in the swapchain.
		let (i, suboptimal, acquire_future) = vulkano::swapchain::acquire_next_image(swapchain.handle.clone(), None).unwrap();

		if suboptimal {
			swapchain.optimal = false;
		}

		let mut rng = rand::thread_rng();
		let clear_values = vec![[0.0, 0.0, 0.0, 1.0].into()];

		// Create command buffer
		// TODO can we avoid creating a comand buffer each render?
		let mut builder = AutoCommandBufferBuilder::primary(self.target.device.clone(), self.queues.graphics.family()).unwrap();
		builder.begin_render_pass(swapchain.framebuffers[i].clone(), false, clear_values).unwrap();

		scene.draw(&self.target, &mut builder, &scene.get(&self.camera).world_projection());

		builder.end_render_pass().unwrap();
		let command_buffer = builder.build().unwrap();

		let device = self.target.device.clone();
		let new_future = self.future.take().unwrap_or_else(|| Box::new(vulkano::sync::now(device)))
			.join(acquire_future)
			.then_execute(self.queues.graphics.clone(), command_buffer).unwrap() // TODO add a semaphore here
			.then_swapchain_present(self.queues.presentation.clone(), swapchain.handle.clone(), i)
			.then_signal_fence_and_flush()
			.unwrap();
		self.future.replace(Box::new(new_future));
	}
}

impl RenderTarget for WindowRenderTarget {
	fn device(&self) -> &Arc<Device> {
		&self.device
	}

	fn render_pass(&self) -> &Arc<dyn RenderPassAbstract + Send + Sync> {
		&self.render_pass
	}

	fn dynamic_state(&self) -> &DynamicState {
		&self.dynamic_state
	}
}

// struct Render(Arc<Object>);
//
// impl bottle::Event for Render {
// 	type Response = ();
// }
//
// impl bottle::Handler<Render> for Window {
// 	fn handle<'a>(mut self: bottle::Receiver<'a, Self>, Render(obj): Render) -> bottle::Output<'a, ()> {
// 		self.render(&obj);
// 		bottle::Output::Now(())
// 	}
// }

impl sync::Worker<Scene> for Window {
	fn cycle(&mut self, state: &Scene) {
		self.render(state);
	}

	fn apply(&mut self, _state: &mut Scene) {
		// ...
	}
}

#[async_std::main]
async fn main() {
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

	let mut scene = Scene::new();
	let camera = scene.new_node(camera::Satellite::new(Matrix4x4::fovx_perspective(
		0.8, // ~ 45°
		800.0/600.0,
		0.1, 10.0
	)));
	// let camera = scene.new_node(camera::Satellite::new(Matrix4x4::orthographic(
	// 	-1.0, 1.0,
	// 	-1.0, 1.0,
	// 	-1.0, 1.0
	// )));
	// let camera = scene.new_node(camera::Satellite::new(Matrix4x4::identity()));
	scene.get_mut(&camera).transformation_mut().translate(Vector3D::new(0.0, 0.0, 2.0));

	let window = Window::new(&event_loop, &physical_device, camera);

	let queue = bottle::EventQueue::new();
	let (loader, loader_queue) = Loader::new(&window.queues.transfert);
	std::thread::spawn(move || {
		async_std::task::block_on(loader_queue.process())
	});

	// Create a cube to render.
	let future_geometry = geometry::Cube::new(0.1, &loader);
	loader.flush();
	let geometry = Arc::new(future_geometry.await);

	let projection = Arc::new(geometry::projection::Standard::new(&window.target.device));
	let material = Arc::new(material::Depth::new(&window.target.device));
	let object = scene.new_root(Object::new(None, geometry, projection, material));

	// scene.get_mut(&object).transformation_mut().translate(Vector3D::new(0.0, 0.0, -2.0));

	// Rendering thread.
	let mut render_thread = sync::Thread::new();
	render_thread.add(window);
	render_thread.start();

	// Conductor.
	let mut conductor = sync::Conductor::new(scene);
	conductor.add(render_thread);

	// Loop.
	event_loop.run(move |event, _, _| {
		match event {
			Event::RedrawRequested(id) => {
				// window.send(Render(object.clone()));
				conductor.inverse_cycle();
			},
			_ => ()
		}
	});
}
