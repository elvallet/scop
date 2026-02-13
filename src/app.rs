use winit::{
	application::ApplicationHandler,
	event::WindowEvent,
	event_loop::ActiveEventLoop,
	window::Window
};
use crate::renderer::{VulkanInstance, VulkanDevice};
use ash::vk;

#[derive(Default)]
pub struct App {
	window: Option<Window>,
	vulkan_instance: Option<VulkanInstance>,
	surface: Option<vk::SurfaceKHR>,
	surface_loader: Option<ash::khr::surface::Instance>,
	device: Option<VulkanDevice>,

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

		self.window = Some(window);
		self.vulkan_instance = Some(vulkan_instance);
		self.surface = Some(surface);
		self.surface_loader = Some(surface_loader);
		self.device = Some(device);
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
				// Recréer swapchain
			}
			WindowEvent::RedrawRequested => {
				// Render frame
			}
			_ => {}
		}	
	}
}

impl Drop for App {
	fn drop(&mut self) {
		unsafe {
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