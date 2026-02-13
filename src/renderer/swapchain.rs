use ash::vk;
use crate::renderer::device::{VulkanDevice};

pub struct VulkanSwapchain {
	pub swapchain_loader: ash::khr::swapchain::Device,
	pub swapchain: vk::SwapchainKHR,
	pub images: Vec<vk::Image>,
	pub image_views: Vec<vk::ImageView>,
	pub format: vk::Format,
	pub extent: vk::Extent2D,
}

impl VulkanSwapchain {
	pub fn new(
		instance: &ash::Instance,
		device: &VulkanDevice,
		surface: vk::SurfaceKHR,
		surface_loader: &ash::khr::surface::Instance,
		window_width: u32,
		window_height: u32
	) -> Result<Self, String> {
		let swapchain_support = VulkanDevice::query_swapchain_support(
			device.physical_device,
			surface,
			surface_loader,
		);

		let surface_format = Self::choose_swap_surface_format(&swapchain_support.formats);
		let present_mode = Self::choose_swap_present_mode(&swapchain_support.present_modes);
		let extent = Self::choose_swap_extent(&swapchain_support.capabilities, window_width, window_height);

		let mut image_count = swapchain_support.capabilities.min_image_count + 1;
		if swapchain_support.capabilities.max_image_count > 0
			&& image_count > swapchain_support.capabilities.max_image_count
		{
			image_count = swapchain_support.capabilities.max_image_count;
		}

		println!("Swapchain will use {} images", image_count);

		let queue_family_indices = [
			device.queue_family_indices.graphics_family.unwrap(),
			device.queue_family_indices.present_family.unwrap(),
		];

		let create_info = if device.queue_family_indices.graphics_family
			!= device.queue_family_indices.present_family
		{
			// Graphics & Present are on two different queues
			vk::SwapchainCreateInfoKHR::default()
				.surface(surface)
				.min_image_count(image_count)
				.image_format(surface_format.format)
				.image_color_space(surface_format.color_space)
				.image_extent(extent)
				.image_array_layers(1)
				.image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
				.image_sharing_mode(vk::SharingMode::CONCURRENT)
				.queue_family_indices(&queue_family_indices)
				.pre_transform(swapchain_support.capabilities.current_transform)
				.composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
				.present_mode(present_mode)
				.clipped(true)
		} else {
			// Graphics & Present are on the same queue
			vk::SwapchainCreateInfoKHR::default()
				.surface(surface)
				.min_image_count(image_count)
				.image_format(surface_format.format)
				.image_color_space(surface_format.color_space)
				.image_extent(extent)
				.image_array_layers(1)
				.image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
				.image_sharing_mode(vk::SharingMode::EXCLUSIVE)
				.pre_transform(swapchain_support.capabilities.current_transform)
				.composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
				.present_mode(present_mode)
				.clipped(true)
		};

		let swapchain_loader = ash::khr::swapchain::Device::new(instance, &device.device);
		let swapchain = unsafe {
			swapchain_loader
				.create_swapchain(&create_info, None)
				.map_err(|e| format!("Failed to create swapchain: {}", e))?
		};

		println!("✓ Swapchain created");

		let images = unsafe {
			swapchain_loader
				.get_swapchain_images(swapchain)
				.map_err(|e| format!("Failed to get swapchain images: {}", e))?
		};

		println!("✓ Got {} swapchain images", images.len());

		let image_views = Self::create_image_views(&device.device, &images, surface_format.format)?;

		println!("✓ Image views created");

		Ok(Self {
			swapchain_loader,
			swapchain,
			images,
			image_views,
			format: surface_format.format,
			extent,
		})
	}

	fn choose_swap_surface_format(
		available_formats: &[vk::SurfaceFormatKHR],
	) -> vk::SurfaceFormatKHR {
		for format in available_formats {
			if format.format == vk::Format::B8G8R8A8_SRGB
				&& format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
			{
				return *format;
			}
		}

		available_formats[0]
	}

	fn choose_swap_present_mode(
		available_present_modes: &[vk::PresentModeKHR],
	) -> vk::PresentModeKHR {
		// Prefer MAILBOX (triple buffering, no tearing, low latency)
		for &mode in available_present_modes {
			if mode == vk::PresentModeKHR::MAILBOX {
				println!("Using MAILBOX present mode (triple buffering)");
				return mode;
			}
		}

		// FIFO always available (eq V-Sync)
		println!("Using FIFO present mode (V-Sync");
		vk::PresentModeKHR::FIFO
	}

	fn choose_swap_extent(
		capabilities: &vk::SurfaceCapabilitiesKHR,
		window_width: u32,
		window_height: u32,
	) -> vk::Extent2D {
		if capabilities.current_extent.width != u32::MAX {
			// already defined
			capabilities.current_extent
		} else {
			// need to define size
			vk::Extent2D {
				width: window_width.clamp(
					capabilities.min_image_extent.width,
					capabilities.max_image_extent.width,
				),
				height: window_height.clamp(
					capabilities.max_image_extent.height,
					capabilities.max_image_extent.height,
				),
			}
		}
	}

	fn create_image_views(
		device: &ash::Device,
		images: &[vk::Image],
		format: vk::Format,
	) -> Result<Vec<vk::ImageView>, String> {
		images
			.iter()
			.map(|&image| {
				let create_info = vk::ImageViewCreateInfo::default()
					.image(image)
					.view_type(vk::ImageViewType::TYPE_2D)
					.format(format)
					.components(vk::ComponentMapping {
						r: vk::ComponentSwizzle::IDENTITY,
						g: vk::ComponentSwizzle::IDENTITY,
						b: vk::ComponentSwizzle::IDENTITY,
						a: vk::ComponentSwizzle::IDENTITY,
					})
					.subresource_range(vk::ImageSubresourceRange {
						aspect_mask: vk::ImageAspectFlags::COLOR,
						base_mip_level: 0,
						level_count: 1,
						base_array_layer: 0,
						layer_count: 1,
					});

				unsafe {
					device
						.create_image_view(&create_info, None)
						.map_err(|e| format!("Failed to create image view: {}", e))
				}
			})
			.collect()
	}

	pub fn recreate(
		&mut self,
		instance: &ash::Instance,
		device: &VulkanDevice,
		surface: vk::SurfaceKHR,
		surface_loader: &ash::khr::surface::Instance,
		window_width: u32,
		window_height: u32,
	) -> Result<(), String> {
		unsafe {
			device
				.device
				.device_wait_idle()
				.map_err(|e| format!("Failed to wait for device idle: {}", e))?;
		}

		self.cleanup(&device.device);

		let new_swapchain = Self::new(
			instance,
			device,
			surface,
			surface_loader,
			window_width,
			window_height,
		)?;

		*self = new_swapchain;

		println!("✓ Swapchain recreated");
		Ok(())
	}

	pub fn cleanup(&mut self, device: &ash::Device) {
		unsafe {
			for &image_view in &self.image_views {
				device.destroy_image_view(image_view, None);
			}
			self.swapchain_loader.destroy_swapchain(self.swapchain, None);
		}
	}
}

impl Drop for VulkanSwapchain {
	fn drop(&mut self) {

	}
}