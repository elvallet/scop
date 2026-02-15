use ash::vk;
use crate::renderer::{
	VulkanDevice, VulkanSwapchain, VulkanRenderPass,
	VulkanPipeline, VulkanCommands, VulkanSync
};

pub struct Renderer {
	commands: VulkanCommands,
	sync:VulkanSync,
}

impl Renderer {
	pub fn new(device: &VulkanDevice) -> Result<Self, String> {
		let commands = VulkanCommands::new(
			&device.device,
			device.queue_family_indices.graphics_family.unwrap(),
			VulkanSync::max_frames_in_flight(),
		)?;

		let sync = VulkanSync::new(&device.device)?;

		Ok(Self { commands, sync })
	}

	pub fn draw_frame(
		&mut self,
		device: &VulkanDevice,
		swapchain: &VulkanSwapchain,
		render_pass: &VulkanRenderPass,
		pipeline: &VulkanPipeline,
	) -> Result<(), String> {
		let current_frame = self.sync.current_frame;

		// 1. Wait for current frame's end
		unsafe {
			device
				.device
				.wait_for_fences(
					&[self.sync.in_flight_fences[current_frame]],
					true,
					u64::MAX
				)
				.map_err(|e| format!("Failed to wait for fence: {}", e))?;
		}

		// 2. Acquire swapchain image
		let (image_index, _is_suboptimal) = unsafe {
			swapchain
				.swapchain_loader
				.acquire_next_image(
					swapchain.swapchain,
					u64::MAX,
					self.sync.image_available_semaphores[current_frame],
					vk::Fence::null(),
				)
				.map_err(|e| format!("Failed to acquire swapchain image: {}", e))?
		};

		// 3. Reset fence
		unsafe {
			device
				.device
				.reset_fences(&[self.sync.in_flight_fences[current_frame]])
				.map_err(|e| format!("Failed to reset fence: {}", e))?;
		}

		// 4. Register commands
		let command_buffer = self.commands.command_buffers[current_frame];

		unsafe {
			device
				.device
				.reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())
				.map_err(|e| format!("Failed to reset command buffer: {}", e))?;
		}

		self.commands.record_command_buffer(
			&device.device,
			command_buffer,
			render_pass.framebuffers[image_index as usize],
			render_pass.render_pass,
			swapchain.extent,
			pipeline.pipeline
		)?;

		// 5. Submit command buffer
		let wait_semaphores = [self.sync.image_available_semaphores[current_frame]];
		let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
		let command_buffers = [command_buffer];
		let signal_semaphores = [self.sync.render_finished_semaphores[current_frame]];

		let submit_info = vk::SubmitInfo::default()
			.wait_semaphores(&wait_semaphores)
			.wait_dst_stage_mask(&wait_stages)
			.command_buffers(&command_buffers)
			.signal_semaphores(&signal_semaphores);

		unsafe {
			device
				.device
				.queue_submit(
					device.graphics_queue,
					&[submit_info],
					self.sync.in_flight_fences[current_frame],
				)
				.map_err(|e| format!("Failed to submit queue: {}", e))?;
		}

		// 6. Present image
		let swapchains = [swapchain.swapchain];
		let image_indices = [image_index];

		let present_info = vk::PresentInfoKHR::default()
			.wait_semaphores(&signal_semaphores)
			.swapchains(&swapchains)
			.image_indices(&image_indices);

		unsafe {
			swapchain
				.swapchain_loader
				.queue_present(device.present_queue, &present_info)
				.map_err(|e| format!("Failed to present: {}", e))?;
		}

		// 7. Move to next frame
		self.sync.current_frame = (current_frame + 1) % VulkanSync::max_frames_in_flight();

		Ok(())
	}

	pub fn cleanup(&self, device: &ash::Device) {
		self.commands.cleanup(device);
		self.sync.cleanup(device);
	}
}

impl Drop for Renderer {
	fn drop(&mut self) {

	}
}