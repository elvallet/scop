use ash::vk::{self, Extent2D};
use std::time::Instant;
use crate::renderer::{
	VulkanDevice, VulkanSwapchain, VulkanRenderPass,
	VulkanPipeline, VulkanCommands, VulkanSync,
	MeshBuffers, UniformBuffers, UniformBufferObject, Descriptors
};
use crate::mesh::Mesh;
use crate::math::{Matrix, Vector, Transform};

pub struct Renderer {
	commands: VulkanCommands,
	sync:VulkanSync,
	mesh_buffers: Option<MeshBuffers>,
	uniform_buffers: UniformBuffers,
	descriptors: Descriptors,
	start_time: Instant
}

impl Renderer {
	pub fn new(
		instance: &ash::Instance,
		device: &VulkanDevice,
		pipeline: &VulkanPipeline,
	) -> Result<Self, String> {
		let commands = VulkanCommands::new(
			&device.device,
			device.queue_family_indices.graphics_family.unwrap(),
			VulkanSync::max_frames_in_flight()
		)?;
		let sync = VulkanSync::new(&device.device)?;

		let uniform_buffers = UniformBuffers::new(instance, device)?;

		let descriptors = Descriptors::new(&device.device, pipeline, &uniform_buffers)?;

		Ok(Self {
			commands,
			sync,
			mesh_buffers: None,
			uniform_buffers,
			descriptors,
			start_time: Instant::now(),
		})
	}

	pub fn load_mesh(
		&mut self,
		instance: &ash::Instance,
		device: &VulkanDevice,
		mesh: &Mesh,
	) -> Result<(), String> {
		if let Some(old_mesh) = &self.mesh_buffers {
			old_mesh.cleanup(&device.device);
		}

		let mesh_buffers = MeshBuffers::from_mesh(
			instance,
			device,
			self.commands.command_pool,
			mesh
		)?;

		self.mesh_buffers = Some(mesh_buffers);

		Ok(())
	}

	fn update_uniform_buffer(
		&self,
		device: &VulkanDevice,
		current_frame: usize,
		extent: Extent2D,
		centroid: [f32; 3],
	) -> Result<(), String> {
		let time = self.start_time.elapsed().as_secs_f32();

		let angle = time * 0.5;

		let to_origin = Transform::translation(-centroid[0], -centroid[1], -centroid[2]);
		let rotation = Transform::rotation_y(angle);
		let from_origin = Transform::translation(centroid[0], centroid[1], centroid[2]);

		let model = from_origin.mul_mat(&rotation).mul_mat(&to_origin);

		let eye = Vector::new(vec![0.0, 0.0, 3.0]);
		let target = Vector::new(vec![centroid[0], centroid[1], centroid[2]]);
		let up = Vector::new(vec![0.0, 1.0, 0.0]);
		let view = Transform::look_at(&eye, &target, &up);

		let aspect = extent.width as f32 / extent.height as f32;
		let proj = crate::math::projection(std::f32::consts::FRAC_PI_4, aspect, 0.1, 100.0);

		let ubo = UniformBufferObject {
			model: matrix_to_array(&model),
			view: matrix_to_array(&view),
			proj: matrix_to_array(&proj),
		};

		self.uniform_buffers.update(&device.device, current_frame, &ubo)
	}

	pub fn draw_frame(
		&mut self,
		device: &VulkanDevice,
		swapchain: &VulkanSwapchain,
		render_pass: &VulkanRenderPass,
		pipeline: &VulkanPipeline,
		centroid: [f32; 3],
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

		// 4. Update uniforms
		self.update_uniform_buffer(device, current_frame, swapchain.extent, centroid)?;

		// 5. Register commands
		let command_buffer = self.commands.command_buffers[current_frame];

		unsafe {
			device
				.device
				.reset_command_buffer(command_buffer, vk::CommandBufferResetFlags::empty())
				.map_err(|e| format!("Failed to reset command buffer: {}", e))?;
		}

		if let Some(mesh_buffers) = &self.mesh_buffers {
			self.commands.record_command_buffer_with_mesh(
				&device.device,
				command_buffer,
				render_pass.framebuffers[image_index as usize],
				render_pass.render_pass,
				swapchain.extent,
				pipeline.pipeline,
				pipeline.pipeline_layout,
				mesh_buffers.vertex_buffer.buffer,
				mesh_buffers.index_buffer.buffer,
				mesh_buffers.index_count,
				self.descriptors.descriptor_sets[current_frame],
			)?;
		} else {
			self.commands.record_command_buffer(
				&device.device,
				command_buffer,
				render_pass.framebuffers[image_index as usize],
				render_pass.render_pass,
				swapchain.extent,
				pipeline.pipeline
			)?;
		}

		// 6. Submit command buffer
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

		// 7. Present image
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
		if let Some(mesh_buffers) = &self.mesh_buffers {
			mesh_buffers.cleanup(device);
		}
		self.uniform_buffers.cleanup(device);
		self.descriptors.cleanup(device);
		self.commands.cleanup(device);
		self.sync.cleanup(device);
	}
}

impl Drop for Renderer {
	fn drop(&mut self) {

	}
}

fn matrix_to_array(m: &Matrix) -> [[f32; 4]; 4] {
    let data = m.as_slice();
    [
        [data[0], data[1], data[2], data[3]],
        [data[4], data[5], data[6], data[7]],
        [data[8], data[9], data[10], data[11]],
        [data[12], data[13], data[14], data[15]],
    ]
}