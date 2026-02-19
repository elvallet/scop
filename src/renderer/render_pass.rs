use ash::vk;

use crate::renderer::DepthBuffer;

pub struct VulkanRenderPass {
	pub render_pass: vk::RenderPass,
	pub framebuffers: Vec<vk::Framebuffer>,
}

impl VulkanRenderPass {
	pub fn new(
		device: &ash::Device,
		swapchain_format: vk::Format,
		swapchain_image_views: &[vk::ImageView],
		swapchain_extent: vk::Extent2D,
		depth_buffer: &DepthBuffer,
	) -> Result<Self, String> {
		let render_pass = Self::create_render_pass(device, swapchain_format)?;

		let framebuffers = Self::create_framebuffers(
			device,
			render_pass,
			swapchain_image_views,
			swapchain_extent,
			depth_buffer
		)?;

		Ok(Self {
			render_pass,
			framebuffers
		})
	}

	fn create_render_pass(
		device: &ash::Device,
		swapchain_format: vk::Format,
	) -> Result<vk::RenderPass, String> {
		let color_attachment = vk::AttachmentDescription::default()
			.format(swapchain_format)
			.samples(vk::SampleCountFlags::TYPE_1)				// no multisampling
			.load_op(vk::AttachmentLoadOp::CLEAR)					// clear at frame's start
			.store_op(vk::AttachmentStoreOp::STORE)				// Store for display
			.stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)		// no stencil
			.stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
			.initial_layout(vk::ImageLayout::UNDEFINED)
			.final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

		let depth_attachment = vk::AttachmentDescription::default()
			.format(DepthBuffer::FORMAT)
			.samples(vk::SampleCountFlags::TYPE_1)
			.load_op(vk::AttachmentLoadOp::CLEAR)
			.store_op(vk::AttachmentStoreOp::DONT_CARE)
			.stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
			.stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
			.initial_layout(vk::ImageLayout::UNDEFINED)
			.final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

		let color_attachment_ref = vk::AttachmentReference::default()
			.attachment(0)
			.layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

		let depth_attachment_ref = vk::AttachmentReference::default()
			.attachment(1)
			.layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

		let subpass = vk::SubpassDescription::default()
			.pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
			.color_attachments(std::slice::from_ref(&color_attachment_ref))
			.depth_stencil_attachment(&depth_attachment_ref);

		// Subpass' dependency
		// It ensures that:
		//  - We wait that the image is available before draw
		//  - We signal that render is done before we present
		let dependency = vk::SubpassDependency::default()
			.src_subpass(vk::SUBPASS_EXTERNAL)
			.dst_subpass(0)
			.src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
			.src_access_mask(vk::AccessFlags::empty())
			.dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
			.dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE);

		let attachments = [color_attachment, depth_attachment];
		let subpasses = [subpass];
		let dependencies = [dependency];

		let render_pass_info = vk::RenderPassCreateInfo::default()
			.attachments(&attachments)
			.subpasses(&subpasses)
			.dependencies(&dependencies);

		let render_pass = unsafe {
			device
				.create_render_pass(&render_pass_info, None)
				.map_err(|e| format!("Failed to create render pass: {}", e))?
		};

		println!("✓ Render pass created");

		Ok(render_pass)
	}

	fn create_framebuffers(
		device: &ash::Device,
		render_pass: vk::RenderPass,
		image_views: &[vk::ImageView],
		extent: vk::Extent2D,
		depth_buffer: &DepthBuffer,
	) -> Result<Vec<vk::Framebuffer>, String> {
		let framebuffers: Result<Vec<_>, _> = image_views
			.iter()
			.map(|&image_view| {
				let attachments = [image_view, depth_buffer.image_view];

				let frambuffer_info = vk::FramebufferCreateInfo::default()
					.render_pass(render_pass)
					.attachments(&attachments)
					.width(extent.width)
					.height(extent.height)
					.layers(1);

				unsafe {
					device
						.create_framebuffer(&frambuffer_info, None)
						.map_err(|e| format!("Failed to create framebuffer: {}", e))
				}
			})
			.collect();

		let framebuffers = framebuffers?;

		println!("✓ Created {} framebuffers", framebuffers.len());

		Ok(framebuffers)
	}

	pub fn recreate_framebuffers(
		&mut self,
		device: &ash::Device,
		image_views: &[vk::ImageView],
		extent: vk::Extent2D,
		depth_buffer: &DepthBuffer
	) -> Result<(), String> {
		for &framebuffer in &self.framebuffers {
			unsafe {
				device.destroy_framebuffer(framebuffer, None);
			}
		}

		self.framebuffers = Self::create_framebuffers(
			device,
			self.render_pass,
			image_views,
			extent,
			depth_buffer
		)?;

		println!("✓ Framebuffers recreated");

		Ok(())
	}

	pub fn cleanup(&self, device: &ash::Device) {
		unsafe {
			for &framebuffer in &self.framebuffers {
				device.destroy_framebuffer(framebuffer, None);
			}
			device.destroy_render_pass(self.render_pass, None);
		}
	}
}

impl Drop for VulkanRenderPass {
	fn drop(&mut self) {
		
	}
}