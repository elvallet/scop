use ash::vk;
use crate::renderer::{VulkanDevice, Buffer};

pub struct DepthBuffer {
	pub image: vk::Image,
	pub image_memory: vk::DeviceMemory,
	pub image_view: vk::ImageView,
}

impl DepthBuffer {
	pub const FORMAT: vk::Format = vk::Format::D32_SFLOAT;

	pub fn new(
		instance: &ash::Instance,
		device: &VulkanDevice,
		width: u32,
		height: u32,
	) -> Result<Self, String> {
		let image_info = vk::ImageCreateInfo::default()
			.image_type(vk::ImageType::TYPE_2D)
			.extent(vk::Extent3D { width, height, depth: 1 })
			.mip_levels(1)
			.array_layers(1)
			.format(Self::FORMAT)
			.tiling(vk::ImageTiling::OPTIMAL)
			.initial_layout(vk::ImageLayout::UNDEFINED)
			.usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
			.samples(vk::SampleCountFlags::TYPE_1)
			.sharing_mode(vk::SharingMode::EXCLUSIVE);

		let image = unsafe {
			device.device.create_image(&image_info, None)
				.map_err(|e| format!("Failed to create depth image: {}", e))?
		};

		let mem_requirements = unsafe {
			device.device.get_image_memory_requirements(image)
		};

		let memory_type = Buffer::find_memory_type(
			instance,
			device.physical_device,
			mem_requirements.memory_type_bits,
			vk::MemoryPropertyFlags::DEVICE_LOCAL,
		)?;

		let alloc_info = vk::MemoryAllocateInfo::default()
			.allocation_size(mem_requirements.size)
			.memory_type_index(memory_type);

		let image_memory = unsafe {
			device.device.allocate_memory(&alloc_info, None)
				.map_err(|e| format!("Failed to allocate depth memory: {}", e))?
		};

		unsafe {
			device.device.bind_image_memory(image, image_memory, 0)
				.map_err(|e| format!("Failed to bind depth memory: {}", e))?;
		}

		let view_info = vk::ImageViewCreateInfo::default()
			.image(image)
			.view_type(vk::ImageViewType::TYPE_2D)
			.format(Self::FORMAT)
			.subresource_range(vk::ImageSubresourceRange {
				aspect_mask: vk::ImageAspectFlags::DEPTH,
				base_mip_level: 0,
				level_count: 1,
				base_array_layer: 0,
				layer_count: 1,
			});


		let image_view = unsafe {
			device.device.create_image_view(&view_info, None)
				.map_err(|e| format!("Failed to create depth image view: {}", e))?
		};

		println!("✓ Depth buffer created ({}x{})", width, height);

		Ok(Self { image, image_memory, image_view })
	}

	pub fn cleanup(&self, device: &ash::Device) {
		unsafe {
			device.destroy_image_view(self.image_view, None);
			device.free_memory(self.image_memory, None);
			device.destroy_image(self.image, None);
		}
	}
}