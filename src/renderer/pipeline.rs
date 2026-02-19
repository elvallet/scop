use ash::vk;
use crate::mesh::Vertex;
use crate::renderer::shader::ShaderModule;

pub struct VulkanPipeline {
	pub pipeline: vk::Pipeline,
	pub pipeline_layout: vk::PipelineLayout,
	pub descriptor_set_layout: vk::DescriptorSetLayout,
}

impl VulkanPipeline {
	pub fn new(
		device: &ash::Device,
		render_pass: vk::RenderPass,
		extent: vk::Extent2D,
	) -> Result<Self, String> {
		// 1. Load shaders
		let vert_shader = ShaderModule::from_file(device, "shaders/shader.vert.spv")?;
		let frag_shader = ShaderModule::from_file(device, "shaders/shader.frag.spv")?;

		// 2. Create descriptor set layout
		let descriptor_set_layout = Self::create_descriptor_set_layout(device)?;

		// 3. Create pipeline layout
		let pipeline_layout = Self::create_pipeline_layout(device, descriptor_set_layout)?;

		// 4. Create graphics pipeline
		let pipeline = Self::create_graphics_pipeline(
			device,
			render_pass,
			pipeline_layout,
			&vert_shader,
			&frag_shader,
			extent
		)?;

		// 5. Cleanup shader modules
		vert_shader.cleanup(device);
		frag_shader.cleanup(device);

		Ok(Self {
			pipeline,
			pipeline_layout,
			descriptor_set_layout,
		})
	}

	fn create_descriptor_set_layout(device: &ash::Device) -> Result<vk::DescriptorSetLayout, String> {
		// Binding 0: Uniform Buffer (MVP matrices)
		let ubo_binding = vk::DescriptorSetLayoutBinding::default()
			.binding(0)
			.descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
			.descriptor_count(1)
			.stage_flags(vk::ShaderStageFlags::VERTEX);

		// Binding 1: Sampler (texture)
		let sampler_binding = vk::DescriptorSetLayoutBinding::default()
			.binding(1)
			.descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
			.descriptor_count(1)
			.stage_flags(vk::ShaderStageFlags::FRAGMENT);

		// Binding 2: Uniform Buffer (mix factor)
		let mix_binding = vk::DescriptorSetLayoutBinding::default()
			.binding(2)
			.descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
			.descriptor_count(1)
			.stage_flags(vk::ShaderStageFlags::FRAGMENT);

		let bindings = [ubo_binding, sampler_binding, mix_binding];

		//let bindings = [ ubo_binding ];

		let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
			.bindings(&bindings);

		let descriptor_set_layout = unsafe {
			device
				.create_descriptor_set_layout(&layout_info, None)
				.map_err(|e| format!("Failed to create descriptor set layout: {}", e))?
		};

		println!("✓ Descriptor set layout created");

		Ok(descriptor_set_layout)
	}

	fn create_pipeline_layout(
		device: &ash::Device,
		descriptor_set_layout: vk::DescriptorSetLayout,
	) -> Result<vk::PipelineLayout, String> {
		let set_layouts = [descriptor_set_layout];

		let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default()
			.set_layouts(&set_layouts);

		let pipeline_layout = unsafe {
			device
				.create_pipeline_layout(&pipeline_layout_info, None)
				.map_err(|e| format!("Failed to create pipeline layout: {}", e))?
		};

		println!("✓ Pipeline layout created");

		Ok(pipeline_layout)
	}

	fn create_graphics_pipeline(
		device: &ash::Device,
		render_pass: vk::RenderPass,
		pipeline_layout: vk::PipelineLayout,
		vert_shader: &ShaderModule,
		frag_shader: &ShaderModule,
		extent: vk::Extent2D,
	) -> Result<vk::Pipeline, String> {
		// ===== SHADER STAGES =====
		let entry_point = c"main";

		let vert_stage = vk::PipelineShaderStageCreateInfo::default()
			.stage(vk::ShaderStageFlags::VERTEX)
			.module(vert_shader.module)
			.name(entry_point);

		let frag_stage = vk::PipelineShaderStageCreateInfo::default()
			.stage(vk::ShaderStageFlags::FRAGMENT)
			.module(frag_shader.module)
			.name(entry_point);

		let shader_stages = [vert_stage, frag_stage];

		// ===== VERTEX INPUT =====
		// Binding description: how to read vertex buffer
		let binding_description = vk::VertexInputBindingDescription::default()
			.binding(0)
			.stride(std::mem::size_of::<Vertex>() as u32)
			.input_rate(vk::VertexInputRate::VERTEX);

		// Attribute description: attributes layout
		let attributes_descriptions = [
			// Position (location = 0)
			vk::VertexInputAttributeDescription::default()
				.binding(0)
				.location(0)
				.format(vk::Format::R32G32B32_SFLOAT)
				.offset(0),
			// TexCoords (location = 1)
			vk::VertexInputAttributeDescription::default()
				.binding(0)
				.location(1)
				.format(vk::Format::R32G32_SFLOAT)
				.offset(12), // 3 floats * 4 bytes
			// Normal (location = 2)
			vk::VertexInputAttributeDescription::default()
				.binding(0)
				.location(2)
				.format(vk::Format::R32G32B32_SFLOAT)
				.offset(20), // 3 + 2 floats * 4 bytes
			// Color (location = 3)
			vk::VertexInputAttributeDescription::default()
				.binding(0)
				.location(3)
				.format(vk::Format::R32G32B32_SFLOAT)
				.offset(32), // 3 + 2 + 3 floats * 4 bytes
		];

		let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default()
			.vertex_binding_descriptions(std::slice::from_ref(&binding_description))
			.vertex_attribute_descriptions(&attributes_descriptions);

		// ===== INPUT ASSEMBLY =====
		let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
			.topology(vk::PrimitiveTopology::TRIANGLE_LIST)
			.primitive_restart_enable(false);

		// ===== VIEWPORT & SCISSORS =====
		let viewport = vk::Viewport::default()
			.x(0.0)
			.y(0.0)
			.width(extent.width as f32)
			.height(extent.height as f32)
			.min_depth(0.0)
			.max_depth(1.0);

		let scissor = vk::Rect2D::default()
			.offset(vk::Offset2D { x: 0, y: 0 })
			.extent(extent);

		let viewport_state = vk::PipelineViewportStateCreateInfo::default()
			.viewports(std::slice::from_ref(&viewport))
			.scissors(std::slice::from_ref(&scissor));

		// ===== RASTERIZATION =====
		let rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
			.depth_clamp_enable(false)
			.rasterizer_discard_enable(false)
			.polygon_mode(vk::PolygonMode::FILL)
			.line_width(1.0)
			.cull_mode(vk::CullModeFlags::NONE)
			.front_face(vk::FrontFace::COUNTER_CLOCKWISE)
			.depth_bias_enable(false);

		// ===== MULTISAMPLING =====
		let multisampling = vk::PipelineMultisampleStateCreateInfo::default()
			.sample_shading_enable(false)
			.rasterization_samples(vk::SampleCountFlags::TYPE_1);

		// ===== COLOR BLENDING =====
		let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
			.color_write_mask(vk::ColorComponentFlags::RGBA)
			.blend_enable(true)
			.src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
			.dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
			.color_blend_op(vk::BlendOp::ADD)
			.src_alpha_blend_factor(vk::BlendFactor::ONE)
			.dst_alpha_blend_factor(vk::BlendFactor::ZERO)
			.alpha_blend_op(vk::BlendOp::ADD);

		let color_blending = vk::PipelineColorBlendStateCreateInfo::default()
			.logic_op_enable(false)
			.attachments(std::slice::from_ref(&color_blend_attachment));

		// ===== DYNAMIC STATE =====
		// Viewport & scissor (resize)
		let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];

		let dynamic_state = vk::PipelineDynamicStateCreateInfo::default()
			.dynamic_states(&dynamic_states);

		// ===== CREATE PIPELINE =====
		let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
			.stages(&shader_stages)
			.vertex_input_state(&vertex_input_info)
			.input_assembly_state(&input_assembly)
			.viewport_state(&viewport_state)
			.rasterization_state(&rasterizer)
			.multisample_state(&multisampling)
			.color_blend_state(&color_blending)
			.dynamic_state(&dynamic_state)
			.layout(pipeline_layout)
			.render_pass(render_pass)
			.subpass(0);

		let pipelines = unsafe {
			device
				.create_graphics_pipelines(vk::PipelineCache::null(),
					std::slice::from_ref(&pipeline_info),
					None,
				)
				.map_err(|e| format!("Failed to create graphics pipeline: {:?}", e.1))?
		};

		println!("✓ Graphics pipeline created");

		Ok(pipelines[0])
	}

	pub fn cleanup(&self, device: &ash::Device) {
		unsafe {
			device.destroy_pipeline(self.pipeline, None);
			device.destroy_pipeline_layout(self.pipeline_layout, None);
			device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);
		}
	}
}

impl Drop for VulkanPipeline {
	fn drop(&mut self) {

	}
}