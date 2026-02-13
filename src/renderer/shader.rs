use ash::vk;
use std::io::Read;

pub struct ShaderModule {
	pub module: vk::ShaderModule,
}

impl ShaderModule {
	pub fn from_file(device: &ash::Device, path: &str) -> Result<Self, String> {
		let mut file = std::fs::File::open(path)
			.map_err(|e| format!("Failed to open shader file {}: {}", path, e))?;

		let mut code = Vec::new();
		file.read_to_end(&mut code)
			.map_err(|e| format!("Failed to red shader file {}: {}", path, e))?;

		let code = Self::align_to_u32(&code);

		let create_info = vk::ShaderModuleCreateInfo::default().code(&code);

		let module = unsafe {
			device
				.create_shader_module(&create_info, None)
				.map_err(|e| format!("Failed to create shader module: {}", e))?
		};

		println!("✓ Shader loaded: {}", path);

		Ok(Self { module })
	}

	fn align_to_u32(data: &[u8]) -> Vec<u32> {
		let mut aligned = Vec::with_capacity(data.len() / 4 + 1);

		for chunk in data.chunks(4) {
			let mut bytes = [0u8; 4];
			bytes[..chunk.len()].copy_from_slice(chunk);
			aligned.push(u32::from_le_bytes(bytes));
		}

		aligned
	}

	pub fn cleanup(&self, device: &ash::Device) {
		unsafe {
			device.destroy_shader_module(self.module, None);
		}
	}
}

impl Drop for ShaderModule {
	fn drop(&mut self) {

	}
}