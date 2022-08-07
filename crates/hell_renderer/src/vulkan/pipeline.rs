use ash::vk;

use super::buffer::VulkanBuffer;
use super::config;
use super::render_pass::VulkanRenderPassData;
use super::shader::VulkanShader;
use super::vertext::VertexInfo;
use super::vulkan_core::VulkanCore;



pub struct VulkanGraphicsPipeline {
    pub render_pass_data: VulkanRenderPassData,
    pub pipeline_layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,

    // TODO: move
    pub vertex_buffer: VulkanBuffer,
    pub index_buffer: VulkanBuffer,
}

impl VulkanGraphicsPipeline {
    // TODO: error handling
    pub fn new(core: & VulkanCore) -> Self {
        let device = &core.device.device;
        let sample_count = vk::SampleCountFlags::TYPE_1;

        let render_pass_data = VulkanRenderPassData::new(core);

        let shader = VulkanShader::new(&core.device.device, config::VERT_SHADER_PATH, config::FRAG_SHADER_PATH);
        let shader_stages = shader.get_stage_create_infos();

        let vertex_info = VertexInfo::new();
        let vertex_input_info = vertex_info.create_input_info();

        let input_assembly = create_pipeline_input_assembly_data();

        let viewport_state_info = core.swapchain.create_pipeline_viewport_data();

        let rasterization_info = create_pipeline_rasterization_data();
        let multisample_state_info = create_multisample_state_date(sample_count);
        let stencil_state = create_stencil_state_data();
        let depth_stencil_info = create_pipeline_depth_stencil_data(stencil_state);

        let color_blend_attachments = [create_color_blend_attachment()];
        let color_blend_info = create_pipeline_blend_data(&color_blend_attachments);

        let pipeline_layout_info = create_pipeline_layout_info_data();
        let pipeline_layout = create_pipeline_layout_data(device, &pipeline_layout_info);

        let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state_info)
            .rasterization_state(&rasterization_info)
            .multisample_state(&multisample_state_info)
            .depth_stencil_state(&depth_stencil_info)
            .color_blend_state(&color_blend_info)
            .layout(pipeline_layout)
            .render_pass(render_pass_data.render_pass.render_pass)
            .subpass(0)
            .base_pipeline_handle(vk::Pipeline::null())
            .base_pipeline_index(-1)
            .build();

        let pipeline = unsafe { device.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None).unwrap()[0] };

        // TODO: move
        let vertex_buffer = VulkanBuffer::from_vertices(core, config::VERTICES);
        let index_buffer = VulkanBuffer::from_indices(core, config::INDICES);

        shader.drop_manual(&core.device.device);

        Self {
            render_pass_data,
            pipeline_layout,
            pipeline,

            vertex_buffer,
            index_buffer
        }
    }

    pub fn recreate_framebuffer(&mut self, core: &VulkanCore) {
        self.render_pass_data.recreate_framebuffer(core);
    }
}

impl VulkanGraphicsPipeline {
    pub fn drop_manual(&self, device: &ash::Device) {
        println!("> dropping VulkanGraphicsPipeline...");

        self.vertex_buffer.drop_manual(device);
        self.index_buffer.drop_manual(device);
        self.render_pass_data.drop_manual(device);

        unsafe {
            device.destroy_pipeline(self.pipeline, None);
            device.destroy_pipeline_layout(self.pipeline_layout, None);
        }
    }
}


fn create_pipeline_input_assembly_data() -> vk::PipelineInputAssemblyStateCreateInfo {
    vk::PipelineInputAssemblyStateCreateInfo::builder()
        .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
        .primitive_restart_enable(false)
        .build()
}

fn create_pipeline_rasterization_data() -> vk::PipelineRasterizationStateCreateInfo {
    vk::PipelineRasterizationStateCreateInfo::builder()
        .depth_clamp_enable(false) // clamp fragments that are beyond the near- and far-plane to them
        .rasterizer_discard_enable(false) // prevetns geometry to pass through te rasterizer stage
        .polygon_mode(vk::PolygonMode::FILL)
        .cull_mode(vk::CullModeFlags::NONE)
        .front_face(vk::FrontFace::CLOCKWISE)
        .depth_bias_enable(false)
        .depth_bias_constant_factor(0.0)
        .depth_bias_clamp(0.0)
        .depth_bias_slope_factor(0.0)
        .line_width(1.0)
        .build()
}

fn create_multisample_state_date(sample_count: vk::SampleCountFlags) -> vk::PipelineMultisampleStateCreateInfo {
    vk::PipelineMultisampleStateCreateInfo::builder()
        .rasterization_samples(sample_count)
        .build()
}

fn create_stencil_state_data() -> vk::StencilOpState {
    vk::StencilOpState {
        fail_op: vk::StencilOp::KEEP,
        pass_op: vk::StencilOp::KEEP,
        depth_fail_op: vk::StencilOp::KEEP,
        compare_op: vk::CompareOp::ALWAYS,
        compare_mask: 0,
        write_mask: 0,
        reference: 0,
    }
}

fn create_pipeline_depth_stencil_data(stencil_state: vk::StencilOpState) -> vk::PipelineDepthStencilStateCreateInfo {
    vk::PipelineDepthStencilStateCreateInfo::builder()
        .depth_test_enable(false)
        .depth_write_enable(false)
        .depth_bounds_test_enable(false)
        .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL)
        .front(stencil_state)
        .back(stencil_state)
        .min_depth_bounds(0.0)
        .max_depth_bounds(1.0)
        .build()
}

fn create_color_blend_attachment() -> vk::PipelineColorBlendAttachmentState  {
    vk::PipelineColorBlendAttachmentState::builder()
        .blend_enable(false)
        .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
        .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
        .alpha_blend_op(vk::BlendOp::ADD)
        .color_write_mask(vk::ColorComponentFlags::RGBA)
        .build()
}
fn create_pipeline_blend_data(color_blend_attachments: &[vk::PipelineColorBlendAttachmentState]) -> vk::PipelineColorBlendStateCreateInfo {

    vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op_enable(false)
        .logic_op(vk::LogicOp::COPY)
        .attachments(color_blend_attachments)
        .blend_constants([0.0, 0.0, 0.0, 0.0])
        .build()
}

// TODO: error handling

fn create_pipeline_layout_info_data() -> vk::PipelineLayoutCreateInfo {
    vk::PipelineLayoutCreateInfo::builder()
        // .set_layouts(set_layouts)
        .push_constant_ranges(&[])
        .build()
}
fn create_pipeline_layout_data(device: &ash::Device, layout_info: &vk::PipelineLayoutCreateInfo /*, set_layouts: &[vk::DescriptorSetLayout]*/) -> vk::PipelineLayout {
    unsafe { device.create_pipeline_layout(layout_info, None).unwrap() }
}
