#![allow(dead_code)]
#![allow(unused)]

use std::borrow::Borrow;
use std::cell::RefCell;

use ash::vk;
use hell_collections::DynArray;
use hell_common::window::HellWindowExtent;
use hell_error::{HellResult, HellError, HellErrorKind, OptToHellErr, ErrToHellErr};
use hell_resources::{ResourceManager, ResourceHandle};
use crate::camera::HellCamera;
use crate::render_types::{RenderData, RenderPackage};
use crate::shader::base_shader::CameraUniform;
use crate::shader::{SpriteShaderSceneData, SpriteShaderGlobalUniformObject, base_shader};
use crate::vulkan::primitives::RenderPassClearFlags;

use super::VulkanContextRef;
use super::primitives::{VulkanSwapchain, VulkanCommands, VulkanCommandBuffer, VulkanRenderPassData, BultinRenderPassType, VulkanImage};
use super::frame::VulkanFrameData;
use super::pipeline::shader_data::{VulkanWorldMesh, VulkanUiMesh};
use super::shader::generic_vulkan_shader::{GenericVulkanShader, NumberFormat};
use super::shader::shader_utils::VulkanUboData;
use super::shader::VulkanSpriteShader;
use hell_core::config;







// ----------------------------------------------------------------------------
// renderer
// ----------------------------------------------------------------------------

pub struct VulkanBackend {
    pub frame_data: VulkanFrameData,
    pub frame_idx: usize,
    pub curr_swap_idx: usize,

    pub cmd_pools: VulkanCommands,
    pub world_meshes: Vec<VulkanWorldMesh>,
    pub ui_meshes: Vec<VulkanUiMesh>,
    pub swapchain: VulkanSwapchain,
    pub render_pass_data: VulkanRenderPassData,

    pub world_shader: VulkanSpriteShader,
    pub test_shader: RefCell<GenericVulkanShader>,

    pub ctx: VulkanContextRef,
}

impl VulkanBackend {
    pub fn new(ctx: VulkanContextRef, swapchain: VulkanSwapchain) -> HellResult<Self> {
        let frame_data = VulkanFrameData::new(&ctx)?;
        let cmds = VulkanCommands::new(&ctx)?;

        let quad_mesh_3d = VulkanWorldMesh::new_quad_3d(&ctx, &cmds)?;
        let world_meshes = vec![quad_mesh_3d];
        let quad_mesh_2d = VulkanUiMesh::new_quad_2d(&ctx, &cmds)?;
        let ui_meshes = vec![quad_mesh_2d];
        let render_pass_data = VulkanRenderPassData::new(&ctx, &swapchain, &cmds)?;

        let world_shader  = VulkanSpriteShader::new(&ctx, &swapchain, config::SPRITE_SHADER_PATH, &render_pass_data)?;

        let test_shader = super::shader::generic_vulkan_shader::GenericVulkanShaderBuilder::new(&ctx, config::TEST_SHADER_PATH)
            .with_global_bindings()
            .with_attribute(NumberFormat::R32G32B32_SFLOAT)
            .with_attribute(NumberFormat::R32G32_SFLOAT)
            .with_global_uniform::<glam::Mat4>("view")
            .with_global_uniform::<glam::Mat4>("proj")
            .with_global_uniform::<glam::Mat4>("view_proj")
            // .with_instance_bindings()
            .build(&swapchain, &render_pass_data.ui_render_pass)?;

        Ok(Self {
            frame_data,
            frame_idx: 0,
            curr_swap_idx: usize::MAX,
            world_meshes,
            ui_meshes,
            swapchain,
            render_pass_data,
            cmd_pools: cmds,
            world_shader,
            test_shader: RefCell::new(test_shader),
            ctx,
        })
    }
}

impl VulkanBackend {
    pub fn recreate_swapchain(&mut self, window_extent: HellWindowExtent) -> HellResult<()> {
        println!("> recreating swapchain...");

        self.swapchain.drop_manual();
        self.swapchain = VulkanSwapchain::new(&self.ctx, window_extent)?;

        Ok(())
    }

    // TODO: improve
    pub fn create_textures(&mut self, resource_manager: &ResourceManager) -> HellResult<()>{
        let texture: HellResult<Vec<_>> = resource_manager.get_all_images().iter()
            .map(|i| VulkanImage::from(&self.ctx, &self.cmd_pools, i))
            .collect();
        let texture = texture?;

        self.world_shader.set_texture_descriptor_sets(texture)?;


        // let texture: HellResult<Vec<_>> = resource_manager.get_all_images().iter()
        //     .map(|i| VulkanImage::from(&self.ctx, &self.cmd_pools, i))
        //     .collect();
        // let texture = texture?;
        //
        // self.test_shader.set_texture_descriptor_sets(texture)?;

        Ok(())
    }
}

// Render-Passes
impl VulkanBackend {
    pub fn begin_render_pass(&self, pass_type: BultinRenderPassType, cmd_buffer: &VulkanCommandBuffer) {
        let (render_pass, frame_buffer) = match pass_type {
            BultinRenderPassType::World => (&self.render_pass_data.world_render_pass, &self.render_pass_data.world_framebuffer),
            BultinRenderPassType::Ui    => (&self.render_pass_data.ui_render_pass, &self.render_pass_data.ui_framebuffer),
        };

        // clear-values
        // -----------
        const MAX_CLEAR_VALUE_COUNT: usize = 2;
        let mut clear_values = DynArray::<vk::ClearValue, MAX_CLEAR_VALUE_COUNT>::from_default();

        let should_clear_color = render_pass.clear_flags.contains(RenderPassClearFlags::COLORBUFFER);
        if should_clear_color {
            clear_values.push(
                vk::ClearValue { color: vk::ClearColorValue { float32: config::CLEAR_COLOR } }
            );
        }

        let should_clear_dpeth = render_pass.clear_flags.contains(RenderPassClearFlags::COLORBUFFER);
        if should_clear_dpeth {
            let should_clear_stencil = render_pass.clear_flags.contains(RenderPassClearFlags::STENCILBUFFER);
            clear_values.push(
                vk::ClearValue { depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: if should_clear_stencil { render_pass.stencil } else { 0 },
                } }
            );
        }

        // render-area
        // -----------
        let render_area = vk::Rect2D {
            offset: vk::Offset2D::default(),
            extent: self.swapchain.extent
        };

        let render_pass_info = vk::RenderPassBeginInfo::builder()
            .render_pass(render_pass.handle)
            .framebuffer(frame_buffer.buffer_at(self.curr_swap_idx))
            .clear_values(clear_values.as_slice())
            .render_area(render_area)
            .build();

        cmd_buffer.cmd_begin_render_pass(&self.ctx, &render_pass_info, vk::SubpassContents::INLINE);
    }

    fn end_renderpass(&self, cmd_buffer: &VulkanCommandBuffer) {
        cmd_buffer.cmd_end_render_pass(&self.ctx);
    }
}



impl VulkanBackend {
    pub fn wait_idle(&self) -> HellResult<()> {
        self.ctx.wait_device_idle()?;
        Ok(())
    }

    pub fn on_window_changed(&mut self, window_extent: HellWindowExtent) -> HellResult<()> {
        self.recreate_swapchain(window_extent)?;
        self.render_pass_data.recreate_framebuffer(&self.ctx, &self.swapchain, &self.cmd_pools)?;
        Ok(())
    }


    #[allow(clippy::modulo_one)]
    pub fn draw_frame(&mut self, _delta_time: f32, render_pkg: &RenderPackage, resources: &ResourceManager) -> HellResult<bool> {
        let device = &self.ctx.device.handle;

        // let frame_data = &self.frame_data[frame_idx];
        let cmd_pool = &self.frame_data.graphics_cmd_pools.get(self.frame_idx).to_render_hell_err()?;
        let in_flight_fence = self.frame_data.in_flight_fence(self.frame_idx);
        in_flight_fence.wait_for_fence(u64::MAX)?;

        let img_available_sem = self.frame_data.img_available_sem(self.frame_idx);
        let (curr_swap_idx, _is_suboptimal) = self.swapchain.aquire_next_image(img_available_sem)?;
        self.curr_swap_idx = curr_swap_idx as usize;

        cmd_pool.reset_cmd_buffer(device, 0)?;
        self.record_cmd_buffer(
            &self.ctx,
            render_pkg,
            resources
        )?;

        // delay resetting the fence until we know for sure we will be submitting work with it (not return early)
        in_flight_fence.reset_fence()?;
        self.submit_queue(self.ctx.device.queues.graphics.queue, &[cmd_pool.get_buffer(0).handle()])?;
        let is_resized = self.present_queue(self.ctx.device.queues.present.queue, &self.swapchain, &[curr_swap_idx])?;

        self.frame_idx = (self.frame_idx + 1) % config::FRAMES_IN_FLIGHT;

        Ok(is_resized)
    }

    pub fn submit_queue(&self, queue: vk::Queue, cmd_buffers: &[vk::CommandBuffer]) -> HellResult<()> {
        let img_available_sems   = [self.frame_data.img_available_sem(self.frame_idx).handle()];
        let render_finished_sems = [self.frame_data.img_render_finished_sem(self.frame_idx).handle()];
        let in_flight_fence      = self.frame_data.in_flight_fence(self.frame_idx).handle();

        let submit_info = [
            vk::SubmitInfo::builder()
                .wait_semaphores(&img_available_sems)
                .wait_dst_stage_mask(&self.frame_data.wait_stages)
                .signal_semaphores(&render_finished_sems)
                .command_buffers(cmd_buffers)
                .build()
        ];

        unsafe { self.ctx.device.handle.queue_submit(queue, &submit_info, in_flight_fence).to_render_hell_err() }
    }

    pub fn present_queue(&self, queue: vk::Queue, swapchain: &VulkanSwapchain, img_indices: &[u32]) -> HellResult<bool> {
        let render_finished_sems = [self.frame_data.img_render_finished_sem(self.frame_idx).handle()];
        let sc = &[swapchain.vk_swapchain];

        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&render_finished_sems)
            .swapchains(sc)
            .image_indices(img_indices)
            .build();

        let result = unsafe { swapchain.swapchain_loader.queue_present(queue, &present_info) };

        // TODO: check
        // do this after queue-present to ensure that the semaphores are in a consistent state - otherwise a signaled semaphore may never be properly waited upon
        let is_resized = match result {
            Ok(is_suboptimal) => { is_suboptimal },
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR | vk::Result::SUBOPTIMAL_KHR) => { true },
            _ => { return Err(HellError::from_msg(HellErrorKind::RenderError, "failed to aquire next image".to_owned())) }
        };

        Ok(is_resized)
    }

    // fn record_cmd_buffer(&self, ctx: &VulkanCtxRef, render_pass_data: &VulkanRenderPassData, swap_img_idx: usize, render_data: &RenderData, resources: &ResourceManager) -> HellResult<()> {
    fn record_cmd_buffer(&self, ctx: &VulkanContextRef, render_pkg: &RenderPackage, resources: &ResourceManager) -> HellResult<()> {
        let device = &ctx.device.handle;
        let cmd_buffer = self.frame_data.get_cmd_buffer(self.frame_idx)?;

        let begin_info = vk::CommandBufferBeginInfo::default();
        cmd_buffer.begin_cmd_buffer(ctx, begin_info)?;

        cmd_buffer.cmd_set_viewport(ctx, 0, &self.swapchain.viewport);
        cmd_buffer.cmd_set_scissor(ctx, 0, &self.swapchain.sissor);

        // world render pass
        self.begin_render_pass(BultinRenderPassType::World, &cmd_buffer);
        self.record_world_cmd_buffer(&cmd_buffer, &render_pkg.world, resources)?;
        self.end_renderpass(&cmd_buffer);

        // ui render pass
        self.update_test_shader()?;
        self.begin_render_pass(BultinRenderPassType::Ui, &cmd_buffer);
        self.record_ui_cmd_buffer(&cmd_buffer, &render_pkg.ui, resources)?;
        self.end_renderpass(&cmd_buffer);

        unsafe { device.end_command_buffer(cmd_buffer.handle())?; }

        Ok(())
    }

    fn record_world_cmd_buffer(&self, cmd_buffer: &VulkanCommandBuffer, render_data: &RenderData, _resources: &ResourceManager) -> HellResult<()> {
        let mut curr_mat_handle = ResourceHandle::MAX;
        let mut curr_mesh_idx = usize::MAX;

        // let mut curr_mat = resources.material_at(0).unwrap();
        let curr_shader = &self.world_shader; // TODO: ...
        let mut curr_mesh = &self.world_meshes[0];
        // let mut curr_shader_key: &str = "";

        // bind static descriptor sets once
        let descriptor_set = [
            curr_shader.get_global_set(0, self.frame_idx)?,
            curr_shader.get_object_set(0, self.frame_idx)?,
        ];

        let min_ubo_alignment = self.ctx.phys_device.device_props.limits.min_uniform_buffer_offset_alignment;
        let dynamic_descriptor_offsets = [
            SpriteShaderSceneData::padded_device_size(min_ubo_alignment) as u32 * self.frame_idx as u32,
        ];

        cmd_buffer.cmd_bind_descriptor_sets(&self.ctx, vk::PipelineBindPoint::GRAPHICS, curr_shader.pipeline.layout, 0, &descriptor_set, &dynamic_descriptor_offsets);

        // TODO: moved here
        cmd_buffer.cmd_bind_pipeline(&self.ctx, vk::PipelineBindPoint::GRAPHICS, curr_shader.pipeline.pipeline);

        // draw each object
        for (idx, rd) in render_data.iter().enumerate() {
            if curr_mat_handle != rd.material {
                curr_mat_handle = rd.material;
                // curr_mat = resources.material_at(curr_mat_handle.id).to_hell_err(HellErrorKind::RenderError)?;

                // bind material descriptors
                let descriptor_set = [ curr_shader.get_material_set(rd.material.id, self.frame_idx)? ];
                cmd_buffer.cmd_bind_descriptor_sets(&self.ctx, vk::PipelineBindPoint::GRAPHICS, curr_shader.pipeline.layout, 2, &descriptor_set, &[]);
            }

            // bind pipeline
            // TODO: remove
            // if curr_shader_key != curr_mat.shader {
            //     curr_shader_key = &curr_mat.shader;
            //     curr_pipeline = &self.pipelines[curr_pipeline_idx];
            //     curr_shader = &self.world_shader;
            //     cmd_buffer.cmd_bind_pipeline(&self.ctx, vk::PipelineBindPoint::GRAPHICS, curr_shader.pipeline.pipeline);
            // }

            // bind mesh
            if curr_mesh_idx != rd.mesh_idx {
                curr_mesh_idx = rd.mesh_idx;
                curr_mesh = &self.world_meshes[curr_mesh_idx];

                let vertex_buffers = [curr_mesh.vertex_buffer.handle];
                cmd_buffer.cmd_bind_vertex_buffers(&self.ctx, 0, &vertex_buffers, &[0]);
                cmd_buffer.cmd_bind_index_buffer(&self.ctx, curr_mesh.index_buffer.handle, 0, VulkanWorldMesh::INDEX_TYPE);
            }

            // bind push constants
            // let push_constants = [
            //     MeshPushConstants {
            //         model: rd.transform.create_model_mat()
            //     }
            // ];

            // let push_const_bytes = std::slice::from_raw_parts(push_constants.as_ptr() as *const u8, std::mem::size_of_val(&push_constants));
            // cmd_buffer.cmd_push_constants(&self.ctx, curr_shader.pipeline.layout, vk::ShaderStageFlags::VERTEX, 0, push_const_bytes);

            // draw
            // value of 'first_instance' is used in the vertex shader to index into the object storage
            cmd_buffer.cmd_draw_indexed(&self.ctx, curr_mesh.indices_count() as u32, 1, 0, 0, idx as u32);
        }

        Ok(())
    }

    fn record_ui_cmd_buffer(&self, cmd_buffer: &VulkanCommandBuffer, render_data: &RenderData, _resources: &ResourceManager) -> HellResult<()> {
        unsafe {
            let mut curr_mat_handle = ResourceHandle::MAX;
            let mut curr_mesh_idx = usize::MAX;

            let shader = &self.test_shader.borrow(); // TODO: ...
            let mut mesh = &self.world_meshes[0];

            // bind static descriptor sets once
            let descriptor_set = [
                shader.global_descriptor(self.frame_idx),
            ];

            cmd_buffer.cmd_bind_descriptor_sets(&self.ctx, vk::PipelineBindPoint::GRAPHICS, shader.pipeline.layout, 0, &descriptor_set, &[]);

            // TODO: moved here
            cmd_buffer.cmd_bind_pipeline(&self.ctx, vk::PipelineBindPoint::GRAPHICS, shader.pipeline.pipeline);

            // draw each object
            for (idx, rd) in render_data.iter().enumerate() {
                // bind mesh
                if curr_mesh_idx != rd.mesh_idx {
                    curr_mesh_idx = rd.mesh_idx;
                    mesh = &self.world_meshes[curr_mesh_idx];

                    let vertex_buffers = [mesh.vertex_buffer.handle];
                    cmd_buffer.cmd_bind_vertex_buffers(&self.ctx, 0, &vertex_buffers, &[0]);
                    cmd_buffer.cmd_bind_index_buffer(&self.ctx, mesh.index_buffer.handle, 0, VulkanWorldMesh::INDEX_TYPE);
                }

                // draw
                // value of 'first_instance' is used in the vertex shader to index into the object storage
                cmd_buffer.cmd_draw_indexed(&self.ctx, mesh.indices_count() as u32, 1, 0, 0, 0);
            }
        }

        Ok(())
    }
}

impl VulkanBackend {
    // TODO: Error handling
    pub fn update_world_shader(&mut self, camera: HellCamera, scene_data: &SpriteShaderSceneData, render_data: &RenderData) -> HellResult<()> {
        let global_uo = SpriteShaderGlobalUniformObject::new(camera.view, camera.proj, camera.view_proj);
        self.world_shader.update_global_uo(global_uo, self.frame_idx)?;
        self.world_shader.update_scene_uo(scene_data, self.frame_idx)?;
        if !render_data.is_empty() {
            self.world_shader.update_object_uo(render_data, self.frame_idx)?;
        }

        Ok(())
    }

    #[allow(unused)]
    pub fn update_test_shader(&self) -> HellResult<()> {
        let cam = HellCamera::new(self.swapchain.aspect_ratio());

        let mut shader = self.test_shader.borrow_mut();

        if let Some(mut uni) = shader.uniform_handle("view") {
            shader.set_uniform(uni, &[cam.view])?;
        }

        if let Some(mut uni) = shader.uniform_handle("proj") {
            shader.set_uniform(uni, &[cam.proj])?;
        }

        if let Some(mut uni) = shader.uniform_handle("view_proj") {
            shader.set_uniform(uni, &[cam.view_proj])?;
        }

        shader.apply_globals(self.frame_idx);

        Ok(())
    }
}
