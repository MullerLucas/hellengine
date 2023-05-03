mod shader;
use hell_core::{error::{HellResult, HellErrorKind, HellError}, collections::dyn_array::DynArray};
pub use shader::*;
pub mod shader_data;



use ash::vk;
use crate::{vulkan::VulkanContextRef, config};

use super::{primitives::{VulkanSwapchain, VulkanRenderPass}, shader_program::PushConstantInfo};





// ----------------------------------------------------------------------------
// command-pools
// ----------------------------------------------------------------------------

#[derive(Debug)]
pub struct VulkanPipeline {
    ctx: VulkanContextRef,
    pub layout: vk::PipelineLayout,
    pub pipeline: vk::Pipeline,
}

impl Drop for VulkanPipeline {
    fn drop(&mut self) {
        println!("> dropping GraphicsPipeline...");

        unsafe {
            let device = &self.ctx.device.handle;
            device.destroy_pipeline(self.pipeline, None);
            device.destroy_pipeline_layout(self.layout, None);
        }
    }
}

impl VulkanPipeline {
    #[allow(clippy::too_many_arguments)]
    pub fn new(ctx: &VulkanContextRef, swapchain: &VulkanSwapchain, shader: VulkanShader, render_pass: &VulkanRenderPass, vert_binding_desc: &[vk::VertexInputBindingDescription], vert_attrb_desc: &[vk::VertexInputAttributeDescription], descriptor_set_layouts: &[vk::DescriptorSetLayout], push_constant_infos: &[PushConstantInfo], depth_test_enabled: bool, is_wireframe: bool) -> HellResult<Self> {
        let device = &ctx.device.handle;
        let sample_count = vk::SampleCountFlags::TYPE_1;

        // shader
        // ------
        let shader_stages = shader.get_stage_create_infos();

        // vertices
        // --------
        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(vert_binding_desc)
            .vertex_attribute_descriptions(vert_attrb_desc)
            .build();

        // input assembly
        // --------------
        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false)
            .build();

        // rasterizer
        // ----------
        let polygin_mode = if is_wireframe { vk::PolygonMode::LINE } else { vk::PolygonMode::FILL };
        let rasterization_info = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false) // clamp fragments that are beyond the near- and far-plane to them
            .rasterizer_discard_enable(false) // prevetns geometry to pass through te rasterizer stage
            .polygon_mode(polygin_mode)
            // TODO:
            .cull_mode(vk::CullModeFlags::NONE)
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
        // TODO: find better pattern to use
        let mut depth_stencil_info = Default::default();
        if depth_test_enabled {
            depth_stencil_info = vk::PipelineDepthStencilStateCreateInfo::builder()
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
        }

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
        let mut push_constants: DynArray<vk::PushConstantRange, {config::VULKAN_SHADER_MAX_PUSH_CONSTANTS}> = DynArray::default();
        for pcr in push_constant_infos {
            push_constants.push(vk::PushConstantRange::builder()
                .offset(pcr.range.offset as u32)
                .size(pcr.range.range as u32)
                .stage_flags(vk::ShaderStageFlags::ALL_GRAPHICS) // TODO: make selectable
                .build())
        }

        // dyn-state
        // ---------
        let dyn_states = [
            vk::DynamicState::VIEWPORT,
            vk::DynamicState::SCISSOR,
            // vk::DynamicState::LINE_WIDTH,
        ];
        let dyn_state_create_info = vk::PipelineDynamicStateCreateInfo::builder()
            .dynamic_states(&dyn_states)
            .build();

        // viewport
        // --------
        // NOTE: even though we are using dyn-state, we still have to set the initial viewport state
        let viewport_state_info = swapchain.create_pipeline_viewport_data();


        // pipeline layout
        // ---------------
        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(descriptor_set_layouts)
            .push_constant_ranges(push_constants.as_slice())
            .build();

        let pipeline_layout = unsafe { device.create_pipeline_layout(&pipeline_layout_info, None) }?;

        // pipeline creation
        // -----------------
        let mut pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly)
            .rasterization_state(&rasterization_info)
            .multisample_state(&multisample_state_info)
            .color_blend_state(&color_blend_info)
            .layout(pipeline_layout)
            .render_pass(render_pass.handle)
            .subpass(0)
            .base_pipeline_handle(vk::Pipeline::null())
            .base_pipeline_index(-1)
            .viewport_state(&viewport_state_info)
            .dynamic_state(&dyn_state_create_info);
        if depth_test_enabled {
            pipeline_info = pipeline_info.depth_stencil_state(&depth_stencil_info);
        }
        let pipeline_info = pipeline_info.build();

        let pipeline = unsafe {
            device.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
                .map_err(|_| HellError::from_msg(HellErrorKind::RenderError, "failed to create graphics pipeline".to_owned()))?
                [0]
        };

        Ok(Self {
            ctx: ctx.clone(),
            layout: pipeline_layout,
            pipeline,
        })
    }
}
