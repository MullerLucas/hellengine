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
    pub fn new(core: &VulkanCore) -> Self {
        let device = &core.device.device;

        let render_pass_data = VulkanRenderPassData::new(core);

        let shader = VulkanShader::new(&core.device.device, config::VERT_SHADER_PATH, config::FRAG_SHADER_PATH);
        let shader_stages = shader.get_stage_create_infos();

        let vertex_info = VertexInfo::new();
        let vertex_input_info = vertex_info.create_input_info();

        let input_assembly = create_pipeline_input_assembly_data();

        let (_, _, viewport_state_info) = core.swapchain.create_pipeline_viewport_data();

        let rasterization_info = create_pipeline_rasterization_data();
        let depth_stencil_info = create_pipeline_depth_stencil_data();

        let (_, color_blend_info) = create_pipeline_blend_data();

        let (_, pipeline_layout) = create_pipeline_layout_data(device);

        let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state_info)
            .rasterization_state(&rasterization_info)
            .depth_stencil_state(&depth_stencil_info)
            .color_blend_state(&color_blend_info)
            .layout(pipeline_layout)
            .render_pass(render_pass_data.render_pass.pass)
            .subpass(0)
            .build();

        let pipeline = unsafe { device.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None).unwrap()[0] };

        // TODO: move
        let vertex_buffer = VulkanBuffer::from_vertices(core, &config::VERTICES);
        let index_buffer = VulkanBuffer::from_indices(core, &config::INDICES);

        Self {
            render_pass_data,
            pipeline_layout,
            pipeline,

            vertex_buffer,
            index_buffer
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
        .cull_mode(vk::CullModeFlags::BACK)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .depth_bias_enable(false)
        .depth_bias_constant_factor(0.0)
        .depth_bias_clamp(0.0)
        .depth_bias_slope_factor(0.0)
        .line_width(1.0)
        .build()
}

fn create_pipeline_depth_stencil_data() -> vk::PipelineDepthStencilStateCreateInfo {
    vk::PipelineDepthStencilStateCreateInfo::builder()
        .depth_test_enable(false)
        .depth_write_enable(false)
        .depth_bounds_test_enable(false)
        .build()
}

fn create_pipeline_blend_data() -> (vk::PipelineColorBlendAttachmentState, vk::PipelineColorBlendStateCreateInfo) {
    let color_blend_attachment = vk::PipelineColorBlendAttachmentState::builder()
        .blend_enable(false)
        .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
        .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
        .alpha_blend_op(vk::BlendOp::ADD)
        .color_write_mask(vk::ColorComponentFlags::RGBA)
        .build();

    let color_blend_info = vk::PipelineColorBlendStateCreateInfo::builder()
        .logic_op_enable(false)
        .logic_op(vk::LogicOp::COPY)
        .attachments(&[color_blend_attachment])
        .blend_constants([0.0, 0.0, 0.0, 0.0])
        .build();

    (color_blend_attachment, color_blend_info)
}

// TODO: error handling
fn create_pipeline_layout_data(device: &ash::Device /*, set_layouts: &[vk::DescriptorSetLayout]*/) -> (vk::PipelineLayoutCreateInfo, vk::PipelineLayout) {
    let layout_info = vk::PipelineLayoutCreateInfo::builder()
        // .set_layouts(set_layouts)
        .push_constant_ranges(&[])
        .build();

    let layout = unsafe { device.create_pipeline_layout(&layout_info, None). unwrap() };

    (layout_info, layout)
}
