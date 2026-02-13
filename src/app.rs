use winit::{
	application::ApplicationHandler,
	event::WindowEvent,
	event_loop::ActiveEventLoop,
	window::Window
};
use crate::renderer::{VulkanInstance, VulkanDevice, VulkanSwapchain};
use ash::vk;

pub struct App {
	window: Option<Window>,
	vulkan_instance: Option<VulkanInstance>,
	surface: Option<vk::SurfaceKHR>,
	surface_loader: Option<ash::khr::surface::Instance>,
	device: Option<VulkanDevice>,
	swapchain: Option<VulkanSwapchain>,

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

		self.window = Some(window);
		self.vulkan_instance = Some(vulkan_instance);
		self.surface = Some(surface);
		self.surface_loader = Some(surface_loader);
		self.device = Some(device);
		self.swapchain = Some(swapchain);
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
					if let (Some(instance), Some(device), Some(surface), Some(surface_loader), Some(swapchain)) =
						(&self.vulkan_instance, &self.device, self.surface, &self.surface_loader, &mut self.swapchain)
					{
						swapchain.recreate(
							&instance.instance,
							device,
							surface,
							surface_loader,
							size.width,
							size.height,
						).expect("Failed to recreate swapchain");
					}
				}
			}
			WindowEvent::RedrawRequested => {
				// Render frame
			}
			_ => {}
		}	
	}
}

impl App {
	fn cleanup(&mut self) {
		unsafe {
			if let Some(device) = &self.device {
				device.device.device_wait_idle().expect("Failed to wait for device idle");
			}

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