use ash::vk;

pub struct VulkanCommands {
	pub command_pool: vk::CommandPool,
	pub command_buffers: Vec<vk::CommandBuffer>,
}

impl VulkanCommands {
	pub fn new(
		device: &ash::Device,
		queue_family_index: u32,
		max_frames_in_flight: usize,
	) -> Result<Self, String> {
		let command_pool = Self::create_command_pool(device, queue_family_index)?;

		let command_buffers = Self::allocate_command_buffers(
			device,
			command_pool,
			max_frames_in_flight,
		)?;

		Ok(Self {
			command_pool,
			command_buffers
		})
	}

	fn create_command_pool(
		device: &ash::Device,
		queue_family_index: u32,
	) -> Result<vk::CommandPool, String> {
		let pool_info = vk::CommandPoolCreateInfo::default()
			.queue_family_index(queue_family_index)
			.flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

		let command_pool = unsafe {
			device
				.create_command_pool(&pool_info, None)
				.map_err(|e| format!("Failed to create command pool: {}", e))?
		};

		println!("✓ Command pool created");

		Ok(command_pool)
	}

	fn allocate_command_buffers(
		device: &ash::Device,
		command_pool: vk::CommandPool,
		count: usize,
	) -> Result<Vec<vk::CommandBuffer>, String> {
		let alloc_info = vk::CommandBufferAllocateInfo::default()
			.command_pool(command_pool)
			.level(vk::CommandBufferLevel::PRIMARY)
			.command_buffer_count(count as u32);

		let command_buffers = unsafe {
			device
				.allocate_command_buffers(&alloc_info)
				.map_err(|e| format!("Failed to allocate command buffers: {}", e))?
		};

		println!("✓ Allocated {} command buffers", count);

		Ok(command_buffers)
	}

	pub fn record_command_buffer(
		&self,
		device: &ash::Device,
		command_buffer: vk::CommandBuffer,
		framebuffer: vk::Framebuffer,
		render_pass: vk::RenderPass,
		extent: vk::Extent2D,
		pipeline: vk::Pipeline,
	) -> Result<(), String> {
		let begin_info = vk::CommandBufferBeginInfo::default();

		unsafe {
			device
				.begin_command_buffer(command_buffer, &begin_info)
				.map_err(|e| format!("Failed to vegin command buffer: {}", e))?;
		}

		let clear_color = vk::ClearValue {
			color: vk::ClearColorValue {
				float32: [0.1, 0.1, 0.15, 1.0],
			},
		};

		let render_pass_info = vk::RenderPassBeginInfo::default()
			.render_pass(render_pass)
			.framebuffer(framebuffer)
			.render_area(vk::Rect2D {
				offset: vk::Offset2D { x: 0, y: 0 },
				extent,
			})
			.clear_values(std::slice::from_ref(&clear_color));

		unsafe {
			device.cmd_begin_render_pass(
				command_buffer,
				&render_pass_info, 
				vk::SubpassContents::INLINE
			);

			device.cmd_bind_pipeline(
				command_buffer,
				vk::PipelineBindPoint::GRAPHICS,
				pipeline
			);

			let viewport = vk::Viewport::default()
				.x(0.0)
				.y(0.0)
				.width(extent.width as f32)
				.height(extent.height as f32)
				.min_depth(0.0)
				.max_depth(1.0);

			device.cmd_set_viewport(command_buffer, 0, std::slice::from_ref(&viewport));

			let scissor = vk::Rect2D::default()
				.offset(vk::Offset2D { x: 0, y: 0 })
				.extent(extent);

			device.cmd_set_scissor(command_buffer, 0, std::slice::from_ref(&scissor));

			// Here: bind vertex / index buffers & draw

			device.cmd_end_render_pass(command_buffer);

			device
				.end_command_buffer(command_buffer)
				.map_err(|e| format!("Failed to end command buffer: {}", e))?;
		}

		Ok(())
	}

	pub fn cleanup(&self, device: &ash::Device) {
		unsafe {
			device.destroy_command_pool(self.command_pool, None);
		}
	}
}

impl Drop for VulkanCommands {
	fn drop(&mut self) {

	}
}