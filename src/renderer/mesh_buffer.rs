use ash::vk;
use crate::mesh::{Mesh, Vertex};
use crate::renderer::{VulkanDevice, buffer::Buffer};

pub struct MeshBuffers {
	pub vertex_buffer: Buffer,
	pub index_buffer: Buffer,
	pub index_count: u32,
}

impl MeshBuffers {
	pub fn from_mesh(
		instance: &ash::Instance,
		device: &VulkanDevice,
		command_pool: vk::CommandPool,
		mesh: &Mesh,
	) -> Result<Self, String> {
		println!("Loading mesh: {} vertices, {} indices", mesh.vertices.len(), mesh.indices.len());

		let vertex_buffer = Self::create_vertex_buffer(
			instance,
			device,
			command_pool,
			&mesh.vertices
		)?;

		let index_buffer = Self::create_index_buffer(
			instance,
			device,
			command_pool,
			&mesh.indices
		)?;

		Ok(Self {
			vertex_buffer,
			index_buffer,
			index_count: mesh.indices.len() as u32,
		})
	}

	fn create_vertex_buffer(
		instance: &ash::Instance,
		device: &VulkanDevice,
		command_pool: vk::CommandPool,
		vertices: &[Vertex],
	) -> Result<Buffer, String> {
		let buffer_size = (std::mem::size_of::<Vertex>() * vertices.len()) as vk::DeviceSize;

		let staging_buffer = Buffer::new(
			instance,
			device,
			buffer_size,
			vk::BufferUsageFlags::TRANSFER_SRC,
			vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
		)?;

		staging_buffer.upload_data(&device.device, vertices)?;

		let vertex_buffer = Buffer::new(
			instance,
			device,
			buffer_size,
			vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
			vk::MemoryPropertyFlags::DEVICE_LOCAL,
		)?;

		Buffer::copy_buffer(
			&device.device,
			command_pool,
			device.graphics_queue,
			staging_buffer.buffer,
			vertex_buffer.buffer,
			buffer_size
		)?;

		staging_buffer.cleanup(&device.device);

		println!("✓ Vertex buffer created ({} bytes)", buffer_size);

		Ok(vertex_buffer)
	}

	fn create_index_buffer(
		instance: &ash::Instance,
		device: &VulkanDevice,
		command_pool: vk::CommandPool,
		indices: &[u32]
	) -> Result<Buffer, String> {
		let buffer_size = (std::mem::size_of::<u32>() * indices.len()) as vk::DeviceSize;

		let staging_buffer = Buffer::new(
			instance,
			device,
			buffer_size,
			vk::BufferUsageFlags::TRANSFER_SRC,
			vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
		)?;

		staging_buffer.upload_data(&device.device, indices)?;

		let index_buffer = Buffer::new(
			instance,
			device,
			buffer_size,
			vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
			vk::MemoryPropertyFlags::DEVICE_LOCAL,
		)?;

		Buffer::copy_buffer(
			&device.device,
			command_pool,
			device.graphics_queue,
			staging_buffer.buffer,
			index_buffer.buffer,
			buffer_size
		)?;

		staging_buffer.cleanup(&device.device);

		println!("✓ Index buffer created ({} bytes)", buffer_size);

		Ok(index_buffer)
	}

	pub fn cleanup(&self, device: &ash::Device) {
		self.vertex_buffer.cleanup(device);
		self.index_buffer.cleanup(device);
	}
}

impl Drop for MeshBuffers {
	fn drop(&mut self) {
		
	}
}