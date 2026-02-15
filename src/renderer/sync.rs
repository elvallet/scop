use ash::vk;

const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub struct VulkanSync {
	pub image_available_semaphores: Vec<vk::Semaphore>,
	pub render_finished_semaphores: Vec<vk::Semaphore>,
	pub in_flight_fences: Vec<vk::Fence>,
	pub current_frame: usize,
}

impl VulkanSync {
	pub fn new(device: &ash::Device) -> Result<Self, String> {
		let mut image_available_semaphores = Vec::new();
		let mut render_finished_semaphores = Vec::new();
		let mut in_flight_fences = Vec::new();

		let semaphore_info = vk::SemaphoreCreateInfo::default();

		let fence_info = vk::FenceCreateInfo::default()
			.flags(vk::FenceCreateFlags::SIGNALED);

		for _ in 0..MAX_FRAMES_IN_FLIGHT {
			unsafe {
				let image_available = device
					.create_semaphore(&semaphore_info, None)
					.map_err(|e| format!("Failed to create semaphore: {}", e))?;

				let render_finished = device
					.create_semaphore(&semaphore_info, None)
					.map_err(|e| format!("Failed to create semaphore: {}", e))?;

				let fence = device
					.create_fence(&fence_info, None)
					.map_err(|e| format!("Failed to create fence: {}", e))?;

				image_available_semaphores.push(image_available);
				render_finished_semaphores.push(render_finished);
				in_flight_fences.push(fence);
			}
		}

		println!("✓ Synchronization objects created");

		Ok(Self {
			image_available_semaphores,
			render_finished_semaphores,
			in_flight_fences,
			current_frame: 0,
		})
	}

	pub fn max_frames_in_flight() -> usize {
		MAX_FRAMES_IN_FLIGHT
	}

	pub fn cleanup(&self, device: &ash::Device) {
		unsafe {
			for i in 0..MAX_FRAMES_IN_FLIGHT {
				device.destroy_semaphore(self.image_available_semaphores[i], None);
				device.destroy_semaphore(self.render_finished_semaphores[i], None);
				device.destroy_fence(self.in_flight_fences[i], None);
			}
		}
	}
}

impl Drop for VulkanSync {
	fn drop(&mut self) {

	}
}