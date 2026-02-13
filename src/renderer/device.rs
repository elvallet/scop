use ash::vk;
use std::ffi::CStr;

pub struct QueueFamilyIndices {
	pub graphics_family: Option<u32>,
	pub present_family: Option<u32>,
}

impl QueueFamilyIndices {
	pub fn is_complete(&self) -> bool {
		self.graphics_family.is_some() && self.present_family.is_some()
	}
}

pub struct VulkanDevice {
	pub physical_device: vk::PhysicalDevice,
	pub device: ash::Device,
	pub graphics_queue: vk::Queue,
	pub present_queue: vk::Queue,
	pub queue_family_indices: QueueFamilyIndices,
}

impl VulkanDevice {
	pub fn new(
		instance: &ash::Instance,
		surface: vk::SurfaceKHR,
		surface_loader: &ash::khr::surface::Instance,
	) -> Result<Self, String> {
		// 1. Select physical device (GPU)
		let physical_device = Self::pick_physical_device(instance, surface, surface_loader)?;

		// 2. Find queue families
		let queue_family_indices = Self::find_queue_families(
			instance,
			physical_device,
			surface,
			surface_loader,
		)?;

		// 3. Create logical device
		let (device, graphics_queue, present_queue) =
			Self::create_logical_device(instance, physical_device, &queue_family_indices)?;

		Ok(Self {
			physical_device,
			device,
			graphics_queue,
			present_queue,
			queue_family_indices,
		})
	}

	fn pick_physical_device(
		instance: &ash::Instance,
		surface: vk::SurfaceKHR,
		surface_loader: &ash::khr::surface::Instance,
	) -> Result<vk::PhysicalDevice, String> {
		let physical_devices = unsafe {
			instance
				.enumerate_physical_devices()
				.map_err(|e| format!("Failed to enumerate physical devices: {}", e))?
		};

		if physical_devices.is_empty() {
			return Err("No Vulkan-capable GPU found".to_string());
		}

		println!("Found {} physical device(s)", physical_devices.len());

		let mut scored_devices: Vec<(vk::PhysicalDevice, u32)> = physical_devices
			.iter()
			.filter_map(|&device| {
				let score = Self::rate_device_suitability(instance, device, surface, surface_loader);
				if score > 0 {
					Some((device, score))
				} else {
					None
				}
			})
			.collect();

		scored_devices.sort_by(|a, b| b.1.cmp(&a.1));

		scored_devices
			.first()
			.map(|(device, score)| {
				let props = unsafe { instance.get_physical_device_properties(*device) };
				let device_name = unsafe { CStr::from_ptr(props.device_name.as_ptr()) };
				println!(
					"✓ Selected GPU: {:?} (score: {})",
					device_name, score
				);
				*device
			})
			.ok_or_else(|| "No suitable GPU found".to_string())
	}

	fn rate_device_suitability(
		instance: &ash::Instance,
		device: vk::PhysicalDevice,
		surface: vk::SurfaceKHR,
		surface_loader: &ash::khr::surface::Instance,
	) -> u32 {
		let props = unsafe { instance.get_physical_device_properties(device) };
		let features = unsafe { instance.get_physical_device_features(device) };

		let mut score = 0;

		if props.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
			score += 1000;
		}

		let mem_props = unsafe { instance.get_physical_device_memory_properties(device) };
		for i in 0..mem_props.memory_heap_count {
			let heap = mem_props.memory_heaps[i as usize];
			if heap.flags.contains(vk::MemoryHeapFlags::DEVICE_LOCAL) {
				score += (heap.size / (1024 * 1024 * 1024)) as u32;
			}
		}

		let queue_families = Self::find_queue_families(instance, device, surface, surface_loader);
		if queue_families.is_err() || !queue_families.unwrap().is_complete() {
			return 0;
		}

		if !Self::check_device_extension_support(instance, device) {
			return 0;
		}

		let surface_support = Self::query_swapchain_support(device, surface, surface_loader);
		if surface_support.formats.is_empty() || surface_support.present_modes.is_empty() {
			return 0;
		}

		if features.geometry_shader == vk::TRUE {
			score += 100;
		}

		score
	}

	fn find_queue_families(
		instance: &ash::Instance,
		device: vk::PhysicalDevice,
		surface: vk::SurfaceKHR,
		surface_loader: &ash::khr::surface::Instance,
	) -> Result<QueueFamilyIndices, String> {
		let queue_families =
			unsafe { instance.get_physical_device_queue_family_properties(device) };

		let mut indices = QueueFamilyIndices {
			graphics_family: None,
			present_family: None
		};

		for (i, queue_family) in queue_families.iter().enumerate() {
			let i = i as u32;

			if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
				indices.graphics_family = Some(i);
			}

			let present_support = unsafe {
				surface_loader
					.get_physical_device_surface_support(device, i, surface)
					.map_err(|e| format!("Failed to check surface surface: {}", e))?
			};

			if present_support {
				indices.present_family = Some(i);
			}

			if indices.is_complete() {
				break;
			}
		}

		if indices.is_complete() {
			Ok(indices)
		} else {
			Err("Could not find suitable queue families".to_string())
		}
	}

	fn check_device_extension_support(
		instance: &ash::Instance,
		device: vk::PhysicalDevice,
	) -> bool {
		let required_extensions = [ash::khr::swapchain::NAME.as_ptr()];

		let available_extensions = unsafe {
			instance
				.enumerate_device_extension_properties(device)
				.unwrap_or_default()
		};

		required_extensions.iter().all(|&required| {
			let required_name = unsafe { CStr::from_ptr(required) };
			available_extensions.iter().any(|ext| {
				let ext_name = unsafe { CStr::from_ptr(ext.extension_name.as_ptr()) };
				ext_name == required_name
			})
		})
	}

	fn create_logical_device(
		instance: &ash::Instance,
		physical_device: vk::PhysicalDevice,
		indices: &QueueFamilyIndices,
	) -> Result<(ash::Device, vk::Queue, vk::Queue), String> {
		let mut unique_queue_families = std::collections::HashSet::new();
		unique_queue_families.insert(indices.graphics_family.unwrap());
		unique_queue_families.insert(indices.present_family.unwrap());

		let queue_properties = [1.0f32];
		let queue_create_infos: Vec<vk::DeviceQueueCreateInfo> = unique_queue_families
			.iter()
			.map(|&queue_family| {
				vk::DeviceQueueCreateInfo::default()
					.queue_family_index(queue_family)
					.queue_priorities(&queue_properties)
			})
			.collect();

		let device_features = vk::PhysicalDeviceFeatures::default();

		let device_extensions = [ash::khr::swapchain::NAME.as_ptr()];

		//let layer_names = if cfg!(debug_assertions) {
		//	vec![c"VK_LAYER_KHRONOS_validation".as_ptr()]
		//} else {
		//	Vec::new()
		//};

		let create_info = vk::DeviceCreateInfo::default()
			.queue_create_infos(&queue_create_infos)
			.enabled_features(&device_features)
			.enabled_extension_names(&device_extensions);
			//.enabled_layer_names(&layer_names);

		let device = unsafe {
			instance
				.create_device(physical_device, &create_info, None)
				.map_err(|e| format!("Failed to create logical device: {}", e))?
		};

		println!("✓ Logical device created");

		let graphics_queue =
			unsafe { device.get_device_queue(indices.graphics_family.unwrap(), 0) };
		let present_queue =
			unsafe { device.get_device_queue(indices.present_family.unwrap(), 0) };

		println!("✓ Queues retrieved");

		Ok((device, graphics_queue, present_queue))
	}

	pub fn query_swapchain_support(
		device: vk::PhysicalDevice,
		surface: vk::SurfaceKHR,
		surface_loader: &ash::khr::surface::Instance
	) -> SwapchainSupportDetails {
		let capabilities = unsafe {
			surface_loader
				.get_physical_device_surface_capabilities(device, surface)
				.expect("Failed to get surface capabilities")
		};

		let formats = unsafe {
			surface_loader
				.get_physical_device_surface_formats(device, surface)
				.expect("Failed to get surface formats")
		};

		let present_modes = unsafe {
			surface_loader
				.get_physical_device_surface_present_modes(device, surface)
				.expect("Failed to get present modes")
		};

		SwapchainSupportDetails { capabilities, formats, present_modes }
	}
}

pub struct SwapchainSupportDetails {
	pub capabilities: vk::SurfaceCapabilitiesKHR,
	pub formats: Vec<vk::SurfaceFormatKHR>,
	pub present_modes: Vec<vk::PresentModeKHR>,
}

impl Drop for VulkanDevice {
	fn drop(&mut self) {
		unsafe {
			self.device.destroy_device(None);
		}
	}
}