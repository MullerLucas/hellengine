#![allow(dead_code)]
#![allow(unused)]

use std::borrow::Borrow;
use std::cell::RefCell;

use ash::vk;
use hell_common::transform::Transform;
use hell_common::window::HellWindowExtent;
use hell_core::collections::dyn_array::DynArray;
use hell_core::error::{HellResult, HellError, HellErrorKind, OptToHellErr, ErrToHellErr};
use crate::camera::HellCamera;
use crate::config;
use crate::render_types::{RenderData, RenderPackage, NumberFormat};
use crate::resources::{TextureManager, MaterialManager, ResourceHandle, ShaderManager};
use crate::vulkan::primitives::RenderPassClearFlags;
use crate::vulkan::shader_program::ShaderProgramBuilder;

use super::shader_program::ShaderProgram;
use super::{VulkanContextRef, VulkanFrame};
use super::primitives::{VulkanSwapchain, VulkanCommands, VulkanCommandBuffer, VulkanRenderPassData, BultinRenderPassType, VulkanImage, VulkanTexture};
use super::pipeline::shader_data::{VulkanWorldMesh, VulkanUiMesh};





// ----------------------------------------------------------------------------
// renderer
// ----------------------------------------------------------------------------

pub type RenderBackend = VulkanBackend;
pub type RenderTexture = VulkanTexture;

pub struct VulkanBackend {
    pub frame: VulkanFrame,
    pub cmds: VulkanCommands,
    pub world_meshes: Vec<VulkanWorldMesh>,
    pub ui_meshes: Vec<VulkanUiMesh>,
    pub swapchain: VulkanSwapchain,
    pub swap_idx: usize,
    pub render_pass_data: VulkanRenderPassData,
    pub ctx: VulkanContextRef,
}

impl VulkanBackend {
    pub fn new(ctx: VulkanContextRef, swapchain: VulkanSwapchain) -> HellResult<Self> {
        let frame = VulkanFrame::new(&ctx)?;
        let cmds = VulkanCommands::new(&ctx)?;
        let quad_mesh_3d = VulkanWorldMesh::new_quad_3d(&ctx, &cmds)?;
        let world_meshes = vec![quad_mesh_3d];
        let quad_mesh_2d = VulkanUiMesh::new_quad_2d(&ctx, &cmds)?;
        let ui_meshes = vec![quad_mesh_2d];
        let render_pass_data = VulkanRenderPassData::new(&ctx, &swapchain, &cmds)?;

        Ok(Self {
            frame,
            world_meshes,
            ui_meshes,
            swapchain,
            swap_idx: usize::MAX,
            render_pass_data,
            cmds,
            // world_shader,

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
        let mut clear_values = DynArray::<vk::ClearValue, MAX_CLEAR_VALUE_COUNT>::default();

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
            .framebuffer(frame_buffer.buffer_at(self.swap_idx))
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
        self.render_pass_data.recreate_framebuffer(&self.ctx, &self.swapchain, &self.cmds)?;
        Ok(())
    }

    pub fn begin_frame(&mut self) -> HellResult<()> {
        self.frame.begin_frame();

        let in_flight_fence = self.frame.in_flight_fence();
        in_flight_fence.wait_for_fence(u64::MAX)?;

        let img_available_sem = self.frame.img_available_sem();
        let (curr_swap_idx, _is_suboptimal) = self.swapchain.aquire_next_image(img_available_sem)?;
        self.swap_idx = curr_swap_idx as usize;

        let cmd_buffer = &self.frame.gfx_cmd_buffer();
        cmd_buffer.reset_cmd_buffer(&self.ctx)?;

        Ok(())
    }

    pub fn draw_frame(&mut self, _delta_time: f32, render_pkg: &RenderPackage, sha_man: &mut ShaderManager, tex_man: &TextureManager, camera: &HellCamera) -> HellResult<()> {
        let ctx = &self.ctx;
        let cmd_buffer = self.frame.gfx_cmd_buffer();

        let begin_info = vk::CommandBufferBeginInfo::default();
        cmd_buffer.begin_cmd_buffer(ctx, begin_info)?;

        cmd_buffer.cmd_set_viewport(ctx, 0, &self.swapchain.viewport);
        cmd_buffer.cmd_set_scissor(ctx, 0, &self.swapchain.sissor);

        // world render pass
        self.update_sprite_shader(sha_man, tex_man, camera, &render_pkg.world)?;
        self.begin_render_pass(BultinRenderPassType::World, &cmd_buffer);
        self.record_generic_cmd_buffer(&cmd_buffer, &render_pkg.world, sha_man, "sprite")?;
        self.end_renderpass(&cmd_buffer);

        // ui render pass
        self.update_test_shader(sha_man, tex_man, &render_pkg.ui)?;
        self.begin_render_pass(BultinRenderPassType::Ui, &cmd_buffer);
        self.record_generic_cmd_buffer(&cmd_buffer, &render_pkg.ui, sha_man, "test")?;
        self.end_renderpass(&cmd_buffer);

        Ok(())
    }

    pub fn end_frame(&mut self) -> HellResult<bool> {
        let ctx = &self.ctx;

        // end cmd-buffer
        let cmd_buffer = &self.frame.gfx_cmd_buffer();
        cmd_buffer.end_command_buffer(ctx)?;
        // reset fence: delay resetting the fence until we know for sure we will be submitting work with it (not return early)
        self.frame.in_flight_fence().reset_fence()?;
        // submit queue
        self.submit_queue(self.ctx.device.queues.graphics.queue, cmd_buffer)?;
        // present queue
        let is_resized = self.present_queue(self.ctx.device.queues.present.queue, &self.swapchain, &[self.swap_idx as u32])?;

        self.frame.end_frame();
        Ok(is_resized)
    }

    pub fn submit_queue(&self, queue: vk::Queue, cmd_buff: &VulkanCommandBuffer) -> HellResult<()> {
        let img_available_sems = [self.frame.img_available_sem().handle()];
        let render_finished_sems = [self.frame.img_render_finished_sem().handle()];
        let in_flight_fence = self.frame.in_flight_fence().handle();
        let cmd_buffers = [cmd_buff.handle()];

        let submit_info = [
            vk::SubmitInfo::builder()
                .wait_semaphores(&img_available_sems)
                .wait_dst_stage_mask(&[self.frame.wait_stages()])
                .signal_semaphores(&render_finished_sems)
                .command_buffers(&cmd_buffers)
                .build()
        ];

        unsafe { self.ctx.device.handle.queue_submit(queue, &submit_info, in_flight_fence).to_render_hell_err() }
    }

    pub fn present_queue(&self, queue: vk::Queue, swapchain: &VulkanSwapchain, img_indices: &[u32]) -> HellResult<bool> {
        let render_finished_sems = [self.frame.img_render_finished_sem().handle()];
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

    fn record_generic_cmd_buffer(&self, cmd_buffer: &VulkanCommandBuffer, render_data: &RenderData, sha_man: &mut ShaderManager, shader: &str) -> HellResult<()> {
        // bind vertex data
        // ----------------
        let mesh = &self.world_meshes[0];
        let vertex_buffers = [mesh.vertex_buffer.handle];
        cmd_buffer.cmd_bind_vertex_buffers(&self.ctx, 0, &vertex_buffers, &[0]);
        cmd_buffer.cmd_bind_index_buffer(&self.ctx, mesh.index_buffer.handle, 0, VulkanWorldMesh::INDEX_TYPE);
        let shader = sha_man.shader_mut(sha_man.handle_res(shader)?);
        cmd_buffer.cmd_bind_pipeline(&self.ctx, vk::PipelineBindPoint::GRAPHICS, shader.pipeline.pipeline);

        // draw each object
        for (idx, rd) in render_data.iter().enumerate() {
            shader.set_local_idx(&self.frame, idx as u32)?;

            // value of 'first_instance' is used in the vertex shader to index into the object storage
            cmd_buffer.cmd_draw_indexed(&self.ctx, mesh.indices_count() as u32, 1, 0, 0, 0);
        }

        Ok(())
    }
}

impl VulkanBackend {
    pub fn update_sprite_shader(&self, sha_man: &mut ShaderManager, tex_man: &TextureManager, camera: &HellCamera, render_data: &RenderData) -> HellResult<()> {
        let shader = sha_man.shader_mut(sha_man.handle_res("sprite")?);

        // global
        // --------
        shader.bind_global();

        shader.set_uniform(
            shader.uniform_handle_res("view")?,
            &[camera.view]
        )?;
        shader.set_uniform(
            shader.uniform_handle_res("proj")?,
            &[camera.proj]
        )?;
        shader.set_uniform(
            shader.uniform_handle_res("view_proj")?,
            &[camera.view_proj]
        )?;
        shader.apply_global_scope(&self.frame, tex_man)?;

        // instance
        // --------
        let dummy = shader.uniform_handle_res("dummy")?;
        shader.bind_instance(0);
        shader.set_uniform(dummy, &[glam::vec4(0.0, 1.0, 0.0, 1.0)])?;
        const TMP_HANDLE: ResourceHandle = ResourceHandle::new(1);
        shader.apply_instance_scope(&self.frame, tex_man, TMP_HANDLE);

        // local
        // -----
        shader.bind_local(0);
        let local_val: Vec<_> = render_data.transforms.iter().map(|t| t.create_model_mat()).collect();
        shader.set_local_storage(&local_val);
        shader.apply_local_scope(&self.frame);

        Ok(())
    }

    #[allow(unused)]
    pub fn update_test_shader(&self, sha_man: &mut ShaderManager, tex_man: &TextureManager, render_data: &RenderData) -> HellResult<()> {
        let cam = HellCamera::new(self.swapchain.aspect_ratio());

        let mut shader = sha_man.shader_mut(sha_man.handle("test").unwrap());

        // --------------------------------------

        shader.bind_global();

        if let Some(mut uni) = shader.uniform_handle("view") {
            shader.set_uniform(uni, &[cam.view])?;
        }

        if let Some(mut uni) = shader.uniform_handle("proj") {
            shader.set_uniform(uni, &[cam.proj])?;
        }

        if let Some(mut uni) = shader.uniform_handle("view_proj") {
            shader.set_uniform(uni, &[cam.view_proj])?;
        }

        shader.apply_global_scope(&self.frame, tex_man)?;

        // --------------------------------------

        if let Some(mut uni) = shader.uniform_handle("shared_color") {
            const ENTYR_0: ResourceHandle = ResourceHandle::new(0);
            shader.bind_shared(0);
            shader.set_uniform(uni, &[glam::vec4(1.0, 0.0, 0.0, 1.0)])?;
            shader.apply_shared_scope(&self.frame, tex_man, ENTYR_0);
        }

        // --------------------------------------

        if let Some(mut uni) = shader.uniform_handle("instance_color") {
            const ENTRY_0: ResourceHandle = ResourceHandle::new(0);
            const ENTRY_1: ResourceHandle = ResourceHandle::new(1);
            shader.bind_instance(0);
            shader.set_uniform(uni, &[glam::vec4(0.0, 1.0, 0.0, 1.0)])?;
            shader.bind_instance(1);
            shader.set_uniform(uni, &[glam::vec4(1.0, 0.2, 1.0, 1.0)])?;
            shader.apply_instance_scope(&self.frame, tex_man, ENTRY_0);
        }

        // --------------------------------------

        shader.bind_local(0);
        let local_val: Vec<_> = render_data.transforms.iter().map(|t| t.create_model_mat()).collect();
        shader.set_local_storage(&local_val);
        shader.apply_local_scope(&self.frame);

        Ok(())
    }
}

impl VulkanBackend {
    pub fn texture_create(&self, data: &[u8], width: usize, height: usize) -> HellResult<VulkanTexture> {
        VulkanTexture::new(&self.ctx, &self.cmds, data, width, height)
    }

    pub fn texture_create_default(&self) -> HellResult<VulkanTexture> {
        VulkanTexture::new_default(&self.ctx, &self.cmds)
    }

    pub fn create_sprite_shader(&self, global_tex: ResourceHandle) -> HellResult<ShaderProgram> {
        let shader = ShaderProgramBuilder::new(&self.ctx, config::SPRITE_SHADER_PATH)
            .with_depth_test()
            .with_attribute(NumberFormat::R32G32B32_SFLOAT)
            .with_attribute(NumberFormat::R32G32_SFLOAT)
            .with_global_uniform::<glam::Mat4>("view")
            .with_global_uniform::<glam::Mat4>("proj")
            .with_global_uniform::<glam::Mat4>("view_proj")
            .with_instance_uniform::<glam::Mat4>("dummy")
            .with_instance_sampler("instance_tex_0")?
            .with_local_uniform::<glam::Mat4>("model")
            .build(&self.swapchain, &self.render_pass_data.world_render_pass)?;

        println!("create sprite shader: \n{:#?}", shader);

        Ok(shader)
    }

    pub fn create_test_shader(&self, global_tex: ResourceHandle) -> HellResult<ShaderProgram> {
        let shader = ShaderProgramBuilder::new(&self.ctx, config::TEST_SHADER_PATH)
            .with_attribute(NumberFormat::R32G32B32_SFLOAT)
            .with_attribute(NumberFormat::R32G32_SFLOAT)
            .with_global_uniform::<glam::Mat4>("view")
            .with_global_uniform::<glam::Mat4>("proj")
            .with_global_uniform::<glam::Mat4>("view_proj")
            .with_global_sampler("global_tex", global_tex)?
            .with_shared_uniform::<glam::Vec4>("shared_color")
            .with_instance_uniform::<glam::Vec4>("instance_color")
            .with_instance_sampler("instance_tex")?
            .with_local_uniform::<glam::Mat4>("model")
            .build(&self.swapchain, &self.render_pass_data.ui_render_pass)?;

        println!("create test shader: \n{:#?}", shader);

        Ok(shader)
    }
}
