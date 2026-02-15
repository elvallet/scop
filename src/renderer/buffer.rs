use ash::vk;
use crate::renderer::VulkanDevice;

pub struct Buffer {
	pub buffer: vk::Buffer,
	pub memory: vk::DeviceMemory,
	pub size: vk::DeviceSize,
}

impl Buffer {
	pub fn new(
		instance: &ash::Instance,
		device: &VulkanDevice,
		size: vk::DeviceSize,
		usage: vk::BufferUsageFlags,
		properties: vk::MemoryPropertyFlags,
	) -> Result<Self, String> {
		let buffer_info = vk::BufferCreateInfo::default()
			.size(size)
			.usage(usage)
			.sharing_mode(vk::SharingMode::EXCLUSIVE);

		let buffer = unsafe {
			device.device
				.create_buffer(&buffer_info, None)
				.map_err(|e| format!("Failed to create buffer: {}", e))?
		};

		let mem_requirements = unsafe {
			device.device.get_buffer_memory_requirements(buffer)
		};

		let memory_type_index = Self::find_memory_type(
			instance,
			device.physical_device,
			mem_requirements.memory_type_bits,
			properties
		)?;

		let alloc_info = vk::MemoryAllocateInfo::default()
			.allocation_size(mem_requirements.size)
			.memory_type_index(memory_type_index);

		let memory = unsafe {
			device.device
				.allocate_memory(&alloc_info, None)
				.map_err(|e| format!("Failed to allocate buffer memory: {}", e))?
		};

		unsafe {
			device.device
				.bind_buffer_memory(buffer, memory, 0)
				.map_err(|e| format!("Failed to bind buffer memory: {}", e))?;
		}

		Ok(Self { buffer, memory, size })
	}

	pub fn upload_data<T: std::marker::Copy>(
		&self,
		device: &ash::Device,
		data: &[T],
	) -> Result<(), String> {
		let data_size = (std::mem::size_of::<T>() * data.len()) as vk::DeviceSize;

		if data_size > self.size {
			return Err(format!(
				"Data size ({}) exceeds buffer size ({})",
				data_size, self.size
			));
		}

		unsafe {
			let ptr = device
				.map_memory(self.memory, 0, data_size, vk::MemoryMapFlags::empty())
				.map_err(|e| format!("Failed to map memory: {}", e))?;

			let mut align = ash::util::Align::new(
				ptr,
				std::mem::align_of::<T>() as u64,
				data_size,
			);
			align.copy_from_slice(data);

			device.unmap_memory(self.memory);
		}

		Ok(())
	}

	fn find_memory_type(
		instance: &ash::Instance,
		physical_device: vk::PhysicalDevice,
		type_filter: u32,
		properties: vk::MemoryPropertyFlags
	) -> Result<u32, String> {
		let mem_properties = unsafe {
			instance.get_physical_device_memory_properties(physical_device)
		};

		for i in 0..mem_properties.memory_type_count {
			let has_type = (type_filter & (1 << i)) != 0;
			let has_properties = mem_properties.memory_types[i as usize]
				.property_flags
				.contains(properties);

			if has_type && has_properties {
				return Ok(i);
			}
		}

		Err("Failed to find suitable memory type".to_string())
	}

	pub fn copy_buffer(
		device: &ash::Device,
		command_pool: vk::CommandPool,
		queue: vk::Queue,
		src: vk::Buffer,
		dst: vk::Buffer,
		size: vk::DeviceSize,
	) -> Result<(), String> {
		let alloc_info = vk::CommandBufferAllocateInfo::default()
			.level(vk::CommandBufferLevel::PRIMARY)
			.command_pool(command_pool)
			.command_buffer_count(1);

		let command_buffer = unsafe {
				device.allocate_command_buffers(&alloc_info)
					.map_err(|e| format!("Failed to allocate command buffer: {}", e))?[0]
		};

		let begin_info = vk::CommandBufferBeginInfo::default()
			.flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

		unsafe {
			device.begin_command_buffer(command_buffer, &begin_info)
				.map_err(|e| format!("Failed to begin command buffer: {}", e))?;

			let copy_region = vk::BufferCopy::default().size(size);
			device.cmd_copy_buffer(command_buffer, src, dst, &[copy_region]);

			device.end_command_buffer(command_buffer)
				.map_err(|e| format!("Failed to end command buffer: {}", e))?;
		}

		let submit_info = vk::SubmitInfo::default()
			.command_buffers(std::slice::from_ref(&command_buffer));

		unsafe {
			device.queue_submit(queue, &[submit_info],	vk::Fence::null())
				.map_err(|e| format!("Failed to submit queue: {}", e))?;

			device.queue_wait_idle(queue)
				.map_err(|e| format!("Failed to wait for queue: {}", e))?;

			device.free_command_buffers(command_pool, &[command_buffer]);
		}

		Ok(())
	}

	pub fn cleanup(&self, device: &ash::Device) {
		unsafe {
			device.destroy_buffer(self.buffer, None);
			device.free_memory(self.memory, None);
		}
	}
}

impl Drop for Buffer {
	fn drop(&mut self) {

	}
}