use ash::vk;
use crate::renderer::{VulkanDevice, Buffer};

#[derive(Clone, Copy)]
pub struct Texture {
	pub image: vk::Image,
	pub image_memory: vk::DeviceMemory,
	pub image_view: vk::ImageView,
	pub sampler: vk::Sampler,
	width: u32,
	height: u32,
}

impl Texture {
	pub fn new(
		path: &str,
		instance: &ash::Instance,
		device: &VulkanDevice,
		command_pool: vk::CommandPool,
	) -> Result<Self, String> {
		let img = image::open(path)
			.map_err(|e| format!("Failed to open texture: {}: {}", path, e))?
			.to_rgb8();

		let (width, height) = img.dimensions();
		let pixels = img.into_raw();
		let size = (width as vk::DeviceSize) * (height as vk::DeviceSize) * 4;

		let staging = Buffer::new(
			instance, device, size,
			vk::BufferUsageFlags::TRANSFER_SRC,
			vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT
		)?;
		staging.upload_data(&device.device, &pixels)?;

		let (image, image_memory) = Self::create_image(instance, device, width, height)?;

		Self::transition_layout(
			&device.device, command_pool, device.graphics_queue,
			image,
			vk::ImageLayout::UNDEFINED,
			vk::ImageLayout::TRANSFER_DST_OPTIMAL,
		)?;

		Self::copy_buffer_to_image(
			&device.device, command_pool, device.graphics_queue,
			staging.buffer, image, width, height,
		)?;

		Self::transition_layout(
			&device.device, command_pool, device.graphics_queue,
			image,
			vk::ImageLayout::TRANSFER_DST_OPTIMAL,
			vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
		)?;

		staging.cleanup(&device.device);

		let image_view = Self::create_image_view(&device.device, image)?;
		let sampler = Self::create_sampler(&device.device)?;

		println!("✓ Texture loaded: {} ({}x{})", path, width, height);

		Ok(Self { image, image_memory, image_view, sampler, width, height })
	}

	pub fn cleanup(&self, device: &ash::Device) {
		unsafe {
			device.destroy_sampler(self.sampler, None);
			device.destroy_image_view(self.image_view, None);
			device.free_memory(self.image_memory, None);
			device.destroy_image(self.image, None);
		}
	}

	fn create_image(
		instance: &ash::Instance,
		device: &VulkanDevice,
		width: u32,
		height: u32,
	) -> Result<(vk::Image, vk::DeviceMemory), String> {
		let image_info = vk::ImageCreateInfo::default()
			.image_type(vk::ImageType::TYPE_2D)
			.extent(vk::Extent3D { width, height, depth: 1 })
			.mip_levels(1)
			.array_layers(1)
			.format(vk::Format::R8G8B8A8_SRGB)
			.tiling(vk::ImageTiling::OPTIMAL)
			.initial_layout(vk::ImageLayout::UNDEFINED)
			.usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
			.samples(vk::SampleCountFlags::TYPE_1)
			.sharing_mode(vk::SharingMode::EXCLUSIVE);

		let image = unsafe {
			device.device.create_image(&image_info, None)
				.map_err(|e| format!("Failed to create image: {}", e))?
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
				.map_err(|e| format!("Failed to allocate image memory: {}", e))?
		};

		unsafe {
			device.device.bind_image_memory(image, image_memory, 0)
				.map_err(|e| format!("Failed to bind image memory: {}", e))?;
		}

		Ok((image, image_memory))
	}

	fn transition_layout(
		device: &ash::Device,
		command_pool: vk::CommandPool,
		queue: vk::Queue,
		image: vk::Image,
		old_layout: vk::ImageLayout,
		new_layout: vk::ImageLayout,
	) -> Result<(), String> {
		let cmd = Self::begin_single_time_commands(device, command_pool)?;

		let (src_access, dst_access, src_stage, dst_stage) = match (old_layout, new_layout) {
			(vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL) => (
				vk::AccessFlags::empty(),
				vk::AccessFlags::TRANSFER_WRITE,
				vk::PipelineStageFlags::TOP_OF_PIPE,
				vk::PipelineStageFlags::TRANSFER,
			),
			(vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL) => (
				vk::AccessFlags::TRANSFER_WRITE,
				vk::AccessFlags::SHADER_READ,
				vk::PipelineStageFlags::TRANSFER,
				vk::PipelineStageFlags::FRAGMENT_SHADER,
			),
			_ => return Err("Unsupported layout transition".to_string()),
		};

		let barrier = vk::ImageMemoryBarrier::default()
			.old_layout(old_layout)
			.new_layout(new_layout)
			.src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
			.dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
			.image(image)
			.subresource_range(vk::ImageSubresourceRange {
				aspect_mask: vk::ImageAspectFlags::COLOR,
				base_mip_level: 0,
				level_count: 1,
				base_array_layer: 0,
				layer_count: 1,
			})
			.src_access_mask(src_access)
			.dst_access_mask(dst_access);

		unsafe {
			device.cmd_pipeline_barrier(
				cmd,
				src_stage, dst_stage,
				vk::DependencyFlags::empty(),
				&[], &[],
				std::slice::from_ref(&barrier),
			);
		}

		Self::end_single_time_commands(device, command_pool, queue, cmd)
	}

	fn copy_buffer_to_image(
		device: &ash::Device,
		command_pool: vk::CommandPool,
		queue: vk::Queue,
		buffer: vk::Buffer,
		image: vk::Image,
		width: u32,
		height: u32,
	) -> Result<(), String> {
		let cmd = Self::begin_single_time_commands(device, command_pool)?;

		let region = vk::BufferImageCopy::default()
			.buffer_offset(0)
			.buffer_row_length(0)
			.buffer_image_height(0)
			.image_subresource(vk::ImageSubresourceLayers {
				aspect_mask: vk::ImageAspectFlags::COLOR,
				mip_level: 0,
				base_array_layer: 0,
				layer_count: 1,
			})
			.image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
			.image_extent(vk::Extent3D { width, height, depth: 1 });

		unsafe {
			device.cmd_copy_buffer_to_image(
				cmd,
				buffer,
				image,
				vk::ImageLayout::TRANSFER_DST_OPTIMAL,
				std::slice::from_ref(&region),
			);
		}

		Self::end_single_time_commands(device, command_pool, queue, cmd)
	}

	fn create_image_view(
		device: &ash::Device,
		image: vk::Image,
	) -> Result<vk::ImageView, String> {
		let view_info = vk::ImageViewCreateInfo::default()
			.image(image)
			.view_type(vk::ImageViewType::TYPE_2D)
			.format(vk::Format::R8G8B8A8_SRGB)
			.subresource_range(vk::ImageSubresourceRange {
				aspect_mask: vk::ImageAspectFlags::COLOR,
				base_mip_level: 0,
				level_count: 1,
				base_array_layer: 0,
				layer_count: 1,
			});

		unsafe {
			device.create_image_view(&view_info, None)
				.map_err(|e| format!("Failed to create image view: {}", e))
		}
	}

	fn create_sampler(device: &ash::Device) -> Result<vk::Sampler, String> {
		let sampler_info = vk::SamplerCreateInfo::default()
			.mag_filter(vk::Filter::LINEAR)
			.min_filter(vk::Filter::LINEAR)
			.address_mode_u(vk::SamplerAddressMode::REPEAT)
			.address_mode_v(vk::SamplerAddressMode::REPEAT)
			.address_mode_w(vk::SamplerAddressMode::REPEAT)
			.anisotropy_enable(false)
			.border_color(vk::BorderColor::INT_OPAQUE_BLACK)
			.unnormalized_coordinates(false)
			.compare_enable(false)
			.mipmap_mode(vk::SamplerMipmapMode::LINEAR);

		unsafe {
			device.create_sampler(&sampler_info, None)
				.map_err(|e| format!("Failed to create sampler: {}", e))
		}
	}

	fn begin_single_time_commands(
		device: &ash::Device,
		command_pool: vk::CommandPool,
	) -> Result<vk::CommandBuffer, String> {
		let alloc_info = vk::CommandBufferAllocateInfo::default()
			.level(vk::CommandBufferLevel::PRIMARY)
			.command_pool(command_pool)
			.command_buffer_count(1);

		let cmd = unsafe {
			device.allocate_command_buffers(&alloc_info)
				.map_err(|e| format!("Failed to allocate command buffer: {}", e))?[0]
		};

		let begin_info = vk::CommandBufferBeginInfo::default()
			.flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

		unsafe {
			device.begin_command_buffer(cmd, &begin_info)
				.map_err(|e| format!("Failed to begin command buffer: {}", e))?
		};

		Ok(cmd)
	}

	fn end_single_time_commands(
		device: &ash::Device,
		command_pool: vk::CommandPool,
		queue: vk::Queue,
		cmd: vk::CommandBuffer,
	) -> Result<(), String> {
		unsafe {
			device.end_command_buffer(cmd)
				.map_err(|e| format!("Failed to end command buffer: {}", e))?;

			let submit_info = vk::SubmitInfo::default()
				.command_buffers(std::slice::from_ref(&cmd));

			device.queue_submit(queue, std::slice::from_ref(&submit_info), vk::Fence::null())
				.map_err(|e| format!("Failed to submit: {}", e))?;

			device.queue_wait_idle(queue)
				.map_err(|e| format!("Failed to wait idle: {}", e))?;

			device.free_command_buffers(command_pool, std::slice::from_ref(&cmd));
		}
		Ok(())
	}
}