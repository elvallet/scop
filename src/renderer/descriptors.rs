use ash::vk;
use crate::renderer::{VulkanPipeline, UniformBuffers, sync::VulkanSync};

pub struct Descriptors {
	pub descriptor_pool: vk::DescriptorPool,
	pub descriptor_sets: Vec<vk::DescriptorSet>,
}

impl Descriptors {
	pub fn new(
		device: &ash::Device,
		pipeline: &VulkanPipeline,
		uniform_buffers: &UniformBuffers
	) -> Result<Self, String> {
		let pool_sizes = [
			vk::DescriptorPoolSize {
				ty: vk::DescriptorType::UNIFORM_BUFFER,
				descriptor_count: VulkanSync::max_frames_in_flight() as u32,
			},
		];

		let pool_info = vk::DescriptorPoolCreateInfo::default()
			.pool_sizes(&pool_sizes)
			.max_sets(VulkanSync::max_frames_in_flight() as u32);

		let descriptor_pool = unsafe {
			device
				.create_descriptor_pool(&pool_info, None)
				.map_err(|e| format!("Failed to create descriptor pool: {}", e))?
		};

		let layouts = vec![pipeline.descriptor_set_layout; VulkanSync::max_frames_in_flight()];

		let alloc_info = vk::DescriptorSetAllocateInfo::default()
			.descriptor_pool(descriptor_pool)
			.set_layouts(&layouts);

		let descriptor_sets = unsafe {
			device
				.allocate_descriptor_sets(&alloc_info)
				.map_err(|e| format!("Failed to allocate descriptor sets: {}", e))?
		};

		for i in 0..VulkanSync::max_frames_in_flight() {
			let buffer_info = vk::DescriptorBufferInfo::default()
				.buffer(uniform_buffers.buffers[i].buffer)
				.offset(0)
				.range(uniform_buffers.buffers[i].size);

			let descriptor_writes = [
				vk::WriteDescriptorSet::default()
					.dst_set(descriptor_sets[i])
					.dst_binding(0)
					.dst_array_element(0)
					.descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
					.buffer_info(std::slice::from_ref(&buffer_info)),
			];

			unsafe {
				device.update_descriptor_sets(&descriptor_writes, &[]);
			}
		}

		println!("✓ Descriptor sets created");

		Ok(Self {
			descriptor_pool,
			descriptor_sets
		})
	}

	pub fn cleanup(&self, device: &ash::Device) {
		unsafe {
			device.destroy_descriptor_pool(self.descriptor_pool, None);
		}
	}
}

impl Drop for Descriptors {
	fn drop(&mut self) {
		
	}
}