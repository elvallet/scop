use winit::{
	application::ApplicationHandler,
	event::WindowEvent,
	event_loop::ActiveEventLoop,
	window::Window
};
use crate::renderer::VulkanInstance;

#[derive(Default)]
pub struct App {
	window: Option<Window>,
	vulkan_instance: Option<VulkanInstance>,
}

impl ApplicationHandler for App {
	fn resumed(&mut self, event_loop: &ActiveEventLoop) {
		let window_attributes = Window::default_attributes()
			.with_title("SCOP - Vulkan Renderer")
			.with_inner_size(winit::dpi::LogicalSize::new(1280, 720));
		self.window = Some(event_loop.create_window(window_attributes)
			.map_err(|e| format!("Failed to create window: {}", e)).unwrap());

		println!("✓ Window created");

		self.vulkan_instance = Some(VulkanInstance::new(self.window.as_ref().unwrap()).unwrap());
		println!("✓ Vulkan instance created");
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