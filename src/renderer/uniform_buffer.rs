use ash::vk;
use crate::renderer::{VulkanDevice, buffer::Buffer, sync::VulkanSync};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct UniformBufferObject {
	pub model: [[f32; 4]; 4],
	pub view: [[f32; 4]; 4],
	pub proj: [[f32; 4]; 4],
}

pub struct UniformBuffers {
	pub buffers: Vec<Buffer>,
}

impl UniformBuffers {
	pub fn new(
		instance: &ash::Instance,
		device: &VulkanDevice,
	) -> Result<Self, String> {
		let buffer_size = std::mem::size_of::<UniformBufferObject>() as vk::DeviceSize;

		let mut buffers = Vec::new();

		for _ in 0..VulkanSync::max_frames_in_flight() {
			let buffer = Buffer::new(
				instance,
				device,
				buffer_size,
				vk::BufferUsageFlags::UNIFORM_BUFFER,
				vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
			)?;

			buffers.push(buffer);
		}

		println!("✓ Uniform buffers created");

		Ok(Self { buffers })
	}

	pub fn update(&self, device: &ash::Device, frame_index: usize, ubo: &UniformBufferObject) -> Result<(), String> {
		self.buffers[frame_index].upload_data(device, std::slice::from_ref(ubo))
	}

	pub fn cleanup(&self, device: &ash::Device) {
		for buffer in &self.buffers {
			buffer.cleanup(device);
		}
	}
}

impl Drop for UniformBuffers {
	fn drop(&mut self) {
		
	}
}