use winit::{
	application::ApplicationHandler,
	event::WindowEvent,
	event_loop::ActiveEventLoop,
	window::Window
};
use crate::renderer::{
	Renderer, VulkanDevice, VulkanInstance, VulkanPipeline, VulkanRenderPass, VulkanSwapchain
};
use ash::vk;
use crate::mesh::{Mesh, DominantAxis};
use crate::parser::obj::load_obj;

pub struct App {
	window: Option<Window>,
	vulkan_instance: Option<VulkanInstance>,
	surface: Option<vk::SurfaceKHR>,
	surface_loader: Option<ash::khr::surface::Instance>,
	device: Option<VulkanDevice>,
	swapchain: Option<VulkanSwapchain>,
	render_pass: Option<VulkanRenderPass>,
	pipeline: Option<VulkanPipeline>,
	renderer: Option<Renderer>,
	mesh: Option<Mesh>,
	centroid: [f32; 3],
	dominant_axis: DominantAxis,
}

impl Default for App {
	fn default() -> Self {
		Self {
			window: None,
			vulkan_instance: None,
			surface: None,
			surface_loader: None,
			device: None,
			swapchain: None,
			render_pass: None,
			pipeline: None,
			renderer: None,
			mesh: None,
			centroid: [0.0, 0.0, 0.0],
			dominant_axis: DominantAxis::X,
		}
	}
}

impl ApplicationHandler for App {
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		let window_attributes = Window::default_attributes()
			.with_title("SCOP - Vulkan Renderer")
			.with_inner_size(winit::dpi::LogicalSize::new(1280, 720));

		let window = event_loop
			.create_window(window_attributes)
			.expect("Failed to create window");

		println!("✓ Window created");

		let vulkan_instance = VulkanInstance::new(&window).expect("Failed to create Vulkan instance");
		println!("✓ Vulkan instance created");

		let (surface, surface_loader) = vulkan_instance
			.create_surface(&window)
			.expect("Failed to create surface");

		let device = VulkanDevice::new(&vulkan_instance.instance, surface, &surface_loader)
			.expect("Failed to create device");

		let size = window.inner_size();
		let swapchain = VulkanSwapchain::new(
			&vulkan_instance.instance,
			&device,
			surface,
			&surface_loader,
			size.width,
			size.height,
		)
		.expect("Failed to create swapchain");

		let render_pass = VulkanRenderPass::new(
			&device.device,
			swapchain.format,
			&swapchain.image_views,
			swapchain.extent,
		)
		.expect("Failed to create render pass");

		let pipeline = VulkanPipeline::new(
			&device.device,
			render_pass.render_pass,
			swapchain.extent,
		)
		.expect("Failed to create pipeline");

		let mut renderer = Renderer::new(&vulkan_instance.instance, &device, &pipeline)
			.expect("Failed to create renderer");

		let mesh_path = std::env::args()
			.nth(1)
			.unwrap_or_else(|| "ressources/42.obj".to_string());

		println!("Loading mesh: {}", mesh_path);

		let mut mesh = load_obj(&mesh_path)
			.expect(&format!("Failed to load mesh: {}", mesh_path));

		mesh.normalize();
		let centroid = mesh.compute_centroid();
		println!("Mesh centroid: {:?}", centroid);

		let dominant_axis = mesh.compute_dominant_axis();
		println!("Mesh dominant axis: {:?}", dominant_axis);

		renderer.load_mesh(&vulkan_instance.instance, &device, &mesh)
			.expect("Failed to load mesh into GPU");

		self.window = Some(window);
		self.vulkan_instance = Some(vulkan_instance);
		self.surface = Some(surface);
		self.surface_loader = Some(surface_loader);
		self.device = Some(device);
		self.swapchain = Some(swapchain);
		self.render_pass = Some(render_pass);
		self.pipeline = Some(pipeline);
		self.renderer = Some(renderer);
		self.mesh = Some(mesh);
		self.centroid = centroid;

        if let Some(window) = &self.window {
            window.request_redraw();
        }
	}

	fn window_event(
			&mut self,
			event_loop: &ActiveEventLoop,
			_window_id: winit::window::WindowId,
			event: WindowEvent,
	) {
		match event {
			WindowEvent::CloseRequested => {
				println!("Window close requested");
				event_loop.exit();
			},
			WindowEvent::Resized(size) => {
				println!("Window resized to {:?}", size);

				if size.width > 0 && size.height > 0 {
					self.handle_resize(size.width, size.height);
				}
			}
			WindowEvent::RedrawRequested => {
				self.draw_frame();

				if let Some(window) = &self.window {
					window.request_redraw();
				}
			}
			_ => {}
		}	
	}
}

impl App {
	fn draw_frame(&mut self) {
		if let (Some(device), Some(swapchain), Some(render_pass), Some(pipeline), Some(renderer)) =
			(&self.device, &self.swapchain, &self.render_pass, &self.pipeline, &mut self.renderer)
		{
			if let Err(e) = renderer.draw_frame(device, swapchain, render_pass, pipeline, self.centroid, self.dominant_axis) {
				eprintln!("Failed to draw frame: {}", e);
			}
		}
	}

	fn handle_resize(&mut self, width: u32, height: u32) {
		if let (Some(instance), Some(device), Some(surface),
				Some(surface_loader), Some(swapchain), Some(render_pass),
				Some(pipeline)) =
			(&self.vulkan_instance, &self.device, self.surface, &self.surface_loader, &mut self.swapchain,
				&mut self.render_pass, &mut self.pipeline)
		{
			swapchain.recreate(
				&instance.instance,
				device,
				surface,
				surface_loader,
				width,
				height,
			).expect("Failed to recreate swapchain");

			render_pass.recreate_framebuffers(
				&device.device,
				&swapchain.image_views,
				swapchain.extent
			).expect("Failed to framebuffers");

			pipeline.cleanup(&device.device);
			*pipeline = VulkanPipeline::new(
				&device.device,
				render_pass.render_pass,
				swapchain.extent
			).expect("Failed to recreate pipeline");
		}
	}

	fn cleanup(&mut self) {
		unsafe {
			if let Some(device) = &self.device {
				device.device.device_wait_idle().expect("Failed to wait for device idle");
			}

			if let (Some(renderer), Some(device)) = (&self.renderer, &self.device) {
				renderer.cleanup(&device.device);
			}
			drop(self.renderer.take());

			if let (Some(pipeline), Some(device)) = (&self.pipeline, &self.device) {
				pipeline.cleanup(&device.device);
			}
			drop(self.pipeline.take());

			if let (Some(render_pass), Some(device)) = (&self.render_pass, &self.device) {
				render_pass.cleanup(&device.device);
			}
			drop(self.render_pass.take());

			if let (Some(swapchain), Some(device)) = (&mut self.swapchain, &self.device) {
				swapchain.cleanup(&device.device);
			}
			drop(self.swapchain.take());

			drop(self.device.take());

			if let (Some(_instance), Some(surface)) = (&self.vulkan_instance, self.surface) {
				if let Some(surface_loader) = &self.surface_loader {
					surface_loader.destroy_surface(surface, None);
				}
			}

			drop(self.vulkan_instance.take());
		}
	}
}

impl Drop for App {
	fn drop(&mut self) {
		self.cleanup();
	}
}