use ash::{vk, Entry};
use raw_window_handle::HasDisplayHandle;
use std::ffi::{CStr};

pub struct VulkanInstance {
	pub entry: Entry,
	pub instance: ash::Instance,
	pub debug_utils: Option<DebugUtils>,
}

pub struct DebugUtils {
	loader: ash::ext::debug_utils::Instance,
	messenger: vk::DebugUtilsMessengerEXT,
}

impl VulkanInstance {
	pub fn new(window: &winit::window::Window) -> Result<Self, String> {
		// 1. Load Vulkan
		let entry = unsafe {
			Entry::load().map_err(|e| format!("Failed to load Vulkan: {}", e))?
		};

		// 2. Check Vulkan version
		let app_info = vk::ApplicationInfo::default()
			.application_name(c"SCOP")
			.application_version(vk::make_api_version(0, 1, 0, 0))
			.engine_name(c"No Engine")
			.engine_version(vk::make_api_version(0, 1, 0, 0))
			.api_version(vk::API_VERSION_1_3); 

		// 3. Required extensions
		let mut extensions = ash_window::enumerate_required_extensions(
			window.display_handle().unwrap().as_raw()
		)
		.map_err(|e| format!("Failed to get required extensions: {}", e))?
		.to_vec();

		// Add debug extension in debug mode only
		#[cfg(debug_assertions)]
		extensions.push(ash::ext::debug_utils::NAME.as_ptr());

		// 4. Validation layers (debug only)
		let layer_names = if cfg!(debug_assertions) {
			vec![c"VK_LAYER_KHRONOS_validation".as_ptr()]
		} else {
			Vec::new()
		};

		// 5. Check validation layers
		#[cfg(debug_assertions)]
		Self::check_validation_layer_support(&entry)?;

		// 6. Config debug messenger
		#[cfg(debug_assertions)]
		let mut debug_create_info = Self::populate_debug_messenger_create_info();

		// 7. Create instance
		let create_info = vk::InstanceCreateInfo::default()
			.application_info(&app_info)
			.enabled_extension_names(&extensions)
			.enabled_layer_names(&layer_names);

		#[cfg(debug_assertions)]
		let create_info = create_info.push_next(&mut debug_create_info);

		let instance = unsafe {
			entry
				.create_instance(&create_info, None)
				.map_err(|e| format!("Failed to create instance: {}", e))?
		};

		// 8. Set up debug messenger (debug only)
		#[cfg(debug_assertions)]
		let debug_utils = Some(Self::setup_debug_messenger(&entry, &instance)?);

		#[cfg(not(debug_assertions))]
		let debug_utils = None;

		Ok(Self {
			entry,
			instance,
			debug_utils,
		})
	}

	#[cfg(debug_assertions)]
	fn check_validation_layer_support(entry: &Entry) -> Result<(), String> {
		let available_layers = unsafe {
			entry
				.enumerate_instance_layer_properties()
				.map_err(|e| format!("Failed to enumerate layers: {}", e))?
		};

		let required = c"VK_LAYER_KHRONOS_validation";

		let found = available_layers.iter().any(|layer| {
			let name = unsafe {CStr::from_ptr(layer.layer_name.as_ptr()) };
			name == required
		});

		if found {
			println!("✓ Validation layer found");
			Ok(())
		} else {
			Err("Validation layer VK_KHRONOS_validation not available".to_string())
		}
	}

	#[cfg(debug_assertions)]
	fn populate_debug_messenger_create_info() -> vk::DebugUtilsMessengerCreateInfoEXT<'static> {
		vk::DebugUtilsMessengerCreateInfoEXT::default()
			.message_severity(
				vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
				| vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
			)
			.message_type(
				vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
				| vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
				| vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
			)
			.pfn_user_callback(Some(vulkan_debug_callback))
	}

	#[cfg(debug_assertions)]
	fn setup_debug_messenger(
		entry: &Entry,
		instance: &ash::Instance,
	) -> Result<DebugUtils, String> {
		let debug_info = Self::populate_debug_messenger_create_info();

		let debug_utils_loader = ash::ext::debug_utils::Instance::new(entry, instance);

		let messenger = unsafe {
			debug_utils_loader
				.create_debug_utils_messenger(&debug_info, None)
				.map_err(|e| format!("Failed to create debug messenger: {}", e))?
		};

		println!("✓ Debug messenger created");

		Ok(DebugUtils { loader: debug_utils_loader, messenger })
	}
}

// Callback for validation messages
#[cfg(debug_assertions)]
unsafe extern "system" fn vulkan_debug_callback(
	message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
	message_type: vk::DebugUtilsMessageTypeFlagsEXT,
	p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
	_p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
	let callback_data = *p_callback_data;
	let message = CStr::from_ptr(callback_data.p_message);

	let severity = match message_severity {
		vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => "[VERBOSE]",
		vk::DebugUtilsMessageSeverityFlagsEXT::INFO => "[INFO]",
		vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => "[WARNING]",
		vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => "[ERROR]",
		_ => "[UNKNOWN]",
	};

	let type_ = match message_type {
		vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => "GENERAL",
		vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "VALIDATION",
		vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "PERFORMANCE",
		_ => "UNKNOWN",
	};

	println!("{} [{}] {:?}", severity, type_, message);

	vk::FALSE
}

// Cleanup
impl Drop for VulkanInstance {
	fn drop(&mut self) {
		unsafe {
			#[cfg(debug_assertions)]
			if let Some(debug_utils) = &self.debug_utils {
				debug_utils
					.loader
					.destroy_debug_utils_messenger(debug_utils.messenger, None);
			}

			self.instance.destroy_instance(None);
		}
	}
}