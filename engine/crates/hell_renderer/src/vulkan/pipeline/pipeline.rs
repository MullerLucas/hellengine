use ash::vk;
use hell_error::{HellResult, HellError, HellErrorKind, ErrToHellErr};
use crate::vulkan::{vulkan_backend::MeshPushConstants, VulkanCtx, VulkanRenderPassData, Vertex, VulkanSwapchain};

use super::shader::VulkanShader;


pub struct VulkanPipeline {
    pub layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,
}

impl VulkanPipeline {
    pub fn new(core: &VulkanCtx, swapchain: &VulkanSwapchain, shader: VulkanShader, render_pass_data: &VulkanRenderPassData, descriptor_set_layouts: &[vk::DescriptorSetLayout]) -> HellResult<Self> {
        let device = &core.device.device;
        let sample_count = vk::SampleCountFlags::TYPE_1;

        // let render_pass_data = VulkanRenderPassData::new(core);

        // shader
        // ------
        let shader_stages = shader.get_stage_create_infos();

        // vertices
        // --------
        let vertex_binding_desc = [Vertex::get_binding_desc()];
        let vertex_attr_desc = Vertex::get_attribute_desc();
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(&vertex_binding_desc)
            .vertex_attribute_descriptions(&vertex_attr_desc)
            .build();

        // input assembly
        // --------------
        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false)
            .build();

        // viewport
        // --------
        let viewport_state_info = swapchain.create_pipeline_viewport_data();

        // rasterizer
        // ----------
        let rasterization_info = vk::PipelineRasterizationStateCreateInfo::builder()
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
            .build();

        // multisampling
        // -------------
        let multisample_state_info = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(sample_count)
            .build();

        // depth / stancil
        // ---------------
        let depth_stencil_info = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL)
            // only keep fragments that fall in a specific range
            .depth_bounds_test_enable(false)
            .min_depth_bounds(0.0)
            .max_depth_bounds(1.0)
            .stencil_test_enable(false)
            .front(vk::StencilOpState::default())
            .back(vk::StencilOpState::default())
            .build();

        // blending
        // --------
        let color_blend_attachments = [
            vk::PipelineColorBlendAttachmentState::builder()
                .blend_enable(true)
                .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
                .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                .color_blend_op(vk::BlendOp::ADD)
                .src_alpha_blend_factor(vk::BlendFactor::ONE)
                .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
                .alpha_blend_op(vk::BlendOp::ADD)
                .color_write_mask(vk::ColorComponentFlags::RGBA)
                .build()
        ];

        let color_blend_info = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(&color_blend_attachments)
            .blend_constants([0.0, 0.0, 0.0, 0.0])
            .build();

        // push-constants
        // --------------
        let push_constants = [
            vk::PushConstantRange::builder()
                .offset(0)
                .size(std::mem::size_of::<MeshPushConstants>() as u32)
                .stage_flags(vk::ShaderStageFlags::VERTEX)
                .build()
        ];

        // pipeline layout
        // ---------------
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(descriptor_set_layouts)
            .push_constant_ranges(&push_constants)
            .build();

        let pipeline_layout = unsafe { device.create_pipeline_layout(&pipeline_layout_info, None).to_render_hell_err() }?;

        // pipeline creation
        // -----------------
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

        let pipeline = unsafe {
            device.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
                .map_err(|_| HellError::from_msg(HellErrorKind::RenderError, "failed to create graphics pipeline".to_owned()))?
                [0]
        };

        // cleanup
        // -------
        shader.drop_manual(&core.device.device);

        Ok(Self {
            layout: pipeline_layout,
            pipeline,
        })
    }
}

impl VulkanPipeline {
    pub fn drop_manual(&self, device: &ash::Device) {
        println!("> dropping GraphicsPipeline...");

        unsafe {
            device.destroy_pipeline(self.pipeline, None);
            device.destroy_pipeline_layout(self.layout, None);
        }
    }
}
