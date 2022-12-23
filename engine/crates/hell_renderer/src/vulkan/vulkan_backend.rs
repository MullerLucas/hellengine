use std::collections::{HashMap, HashSet};

use ash::vk;
use hell_collections::DynArray;
use hell_common::transform::Transform;
use hell_common::window::HellWindowExtent;
use hell_core::config;
use hell_error::{HellResult, HellError, HellErrorKind, OptToHellErr};
use hell_resources::{ResourceManager, ResourceHandle};
use crate::render_data::{SceneData, ObjectData, GlobalUniformObject, TmpCamera};
use crate::vulkan::image::TextureImage;

use super::command_buffer::VulkanCommands;
use super::{VulkanCtxRef, VulkanSwapchain};
use super::frame::VulkanFrameData;
use super::pipeline::shader_data::{VulkanUboData, VulkanMesh, MeshPushConstants};
use super::render_pass::{VulkanRenderPassData, RenderPassClearFlags, BultinRenderPassType};
use super::shader::VulkanSpriteShader;







// ----------------------------------------------------------------------------
// render data
// ----------------------------------------------------------------------------

pub struct RenderDataChunk<'a> {
    pub mesh_idx: usize,
    pub transform: &'a Transform,
    pub material: ResourceHandle,
}

pub struct RenderData {
    pub meshes: Vec<usize>,
    pub transforms: Vec<Transform>,
    pub materials: Vec<ResourceHandle>,
}

impl Default for RenderData {
    fn default() -> Self {
        Self {
            meshes: Vec::new(),
            transforms: Vec::new(),
            materials: Vec::new(),
        }
    }
}

impl RenderData {
    pub fn data_count(&self) -> usize {
        self.meshes.len()
    }

    pub fn add_data(&mut self, mesh_idx: usize, material: ResourceHandle, trans: Transform) -> usize {
        self.meshes.push(mesh_idx);
        self.transforms.push(trans);
        self.materials.push(material);

        self.data_count()
    }

    pub fn data_at(&self, idx: usize) -> RenderDataChunk {
        RenderDataChunk {
            mesh_idx: self.meshes[idx],
            transform: &self.transforms[idx],
            material: self.materials[idx]
        }
    }
}

impl RenderData {
    pub fn iter(&self) -> RenderDataIter {
        self.into_iter()
    }
}

impl<'a> IntoIterator for &'a RenderData {
    type Item = RenderDataChunk<'a>;
    type IntoIter = RenderDataIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        RenderDataIter::new(self)
    }
}

pub struct RenderDataIter<'a> {
    idx: usize,
    render_data: &'a RenderData,
}

impl<'a> RenderDataIter<'a> {
    pub fn new(render_data: &'a RenderData) -> RenderDataIter<'a> {
        Self {
            idx: 0,
            render_data,
        }
    }
}

impl<'a> Iterator for RenderDataIter<'a> {
    type Item = RenderDataChunk<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.render_data.data_count() > self.idx {
            let result = Some(self.render_data.data_at(self.idx));
            self.idx += 1;
            result
        } else {
            None
        }
    }
}




// ----------------------------------------------------------------------------
// renderer
// ----------------------------------------------------------------------------

pub struct VulkanBackend {
    pub frame_idx: usize,
    pub frame_data: VulkanFrameData,
    pub cmd_pools: VulkanCommands,
    pub meshes: Vec<VulkanMesh>,
    pub swapchain: VulkanSwapchain,
    pub render_pass_data: VulkanRenderPassData,
    pub shaders: HashMap<String, VulkanSpriteShader>,
    pub ctx: VulkanCtxRef,
}

impl VulkanBackend {
    pub fn new(ctx: VulkanCtxRef, swapchain: VulkanSwapchain) -> HellResult<Self> {
        let frame_data = VulkanFrameData::new(&ctx)?;
        let cmds = VulkanCommands::new(&ctx)?;
        let quad_mesh = VulkanMesh::new_quad(&ctx, &cmds)?;
        let meshes = vec![quad_mesh];
        let render_pass_data = VulkanRenderPassData::new(&ctx, &swapchain, &cmds)?;
        let shaders = HashMap::new();

        Ok(Self {
            frame_idx: 0,
            frame_data,
            shaders,
            meshes,
            swapchain,
            render_pass_data,
            cmd_pools: cmds,
            ctx,
        })
    }
}

impl VulkanBackend {
    pub fn recreate_swapchain(&mut self, window_extent: HellWindowExtent) -> HellResult<()> {
        println!("> recreating swapchain...");

        // self.swapchain.drop_manual(&self.ctx.device.device);
        let swapchain_new = VulkanSwapchain::new(&self.ctx, window_extent)?;
        self.swapchain = swapchain_new;

        Ok(())
    }

    pub fn create_shaders(&mut self, shader_paths: HashSet<String>, resource_manager: &ResourceManager) -> HellResult<()>{
        for path in shader_paths {
            let texture: HellResult<Vec<_>> = resource_manager.get_all_images().iter()
                .map(|i| TextureImage::from(&self.ctx, &self.cmd_pools, i))
                .collect();
            let texture = texture?;

            let shader = VulkanSpriteShader::new(&self.ctx, &self.swapchain, &path, &self.render_pass_data, texture)?;
            self.shaders.insert(path, shader);
        }

        Ok(())
    }
}

// Render-Passes
impl VulkanBackend {
    pub fn begin_render_pass(&self, pass_type: BultinRenderPassType, cmd_buffer: vk::CommandBuffer, swap_img_idx: usize) {
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
            .framebuffer(frame_buffer.buffer_at(swap_img_idx))
            .clear_values(clear_values.as_slice())
            .render_area(render_area)
            .build();

        unsafe { self.ctx.device.device.cmd_begin_render_pass(cmd_buffer, &render_pass_info, vk::SubpassContents::INLINE); }
    }

    fn end_renderpass(&self, cmd_buffer: vk::CommandBuffer) {
        unsafe {
            self.ctx.device.device.cmd_end_render_pass(cmd_buffer);
        }
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

    pub fn draw_frame(&mut self, _delta_time: f32, world_render_data: &RenderData, resources: &ResourceManager) -> HellResult<bool> {
        let device = &self.ctx.device.device;

        // let frame_data = &self.frame_data[frame_idx];
        let cmd_pool = &self.frame_data.graphics_cmd_pools.get(self.frame_idx).to_render_hell_err()?;
        self.frame_data.wait_for_in_flight(&self.ctx.device.device, self.frame_idx)?;

        // TODO: check
        // let swap_img_idx = match self.swapchain.aquire_next_image(frame_data.img_available_sem[0]) {
        //     Ok((idx, _)) => { idx },
        //     Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => { return true },
        //     _ => { panic!("failed to aquire next image") }
        // };
        let (curr_swap_idx, _is_suboptimal) = self.swapchain.aquire_next_image(self.frame_data.img_available_semaphors[self.frame_idx])?;

        cmd_pool.reset_cmd_buffer(device, 0)?;
        self.record_cmd_buffer(
            &self.ctx,
            curr_swap_idx as usize,
            world_render_data,
            resources
        )?;

        // delay resetting the fence until we know for sure we will be submitting work with it (not return early)
        self.frame_data.reset_in_flight_fence(device, self.frame_idx)?;
        self.frame_data.submit_queue(device, self.ctx.device.queues.graphics.queue, &[cmd_pool.get_buffer(0)], self.frame_idx)?;

        let present_result = self.frame_data.present_queue(self.ctx.device.queues.present.queue, &self.swapchain, &[curr_swap_idx], self.frame_idx);

        // TODO: check
        // do this after queue-present to ensure that the semaphores are in a consistent state - otherwise a signaled semaphore may never be properly waited upon
        let is_resized = match present_result {
            Ok(is_suboptimal) => { is_suboptimal },
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR | vk::Result::SUBOPTIMAL_KHR) => { true },
            _ => { return Err(HellError::from_msg(HellErrorKind::RenderError, "failed to aquire next image".to_owned())) }
        };

        self.frame_idx = (self.frame_idx + 1) % config::FRAMES_IN_FLIGHT as usize;

        Ok(is_resized)
    }

    // fn record_cmd_buffer(&self, ctx: &VulkanCtxRef, render_pass_data: &VulkanRenderPassData, swap_img_idx: usize, render_data: &RenderData, resources: &ResourceManager) -> HellResult<()> {
    fn record_cmd_buffer(&self, ctx: &VulkanCtxRef, swap_img_idx: usize, render_data: &RenderData, resources: &ResourceManager) -> HellResult<()> {
        let begin_info = vk::CommandBufferBeginInfo::default();
        let device = &ctx.device.device;
        let cmd_buffer = self.frame_data.get_cmd_buffer(self.frame_idx)?;

        unsafe { device.begin_command_buffer(cmd_buffer, &begin_info)?; }

        // world render pass
        self.begin_render_pass(BultinRenderPassType::World, cmd_buffer, swap_img_idx);
        self.record_scene_cmd_buffer(device, cmd_buffer, render_data, resources)?;
        self.end_renderpass(cmd_buffer);

        // ui render pass
        self.begin_render_pass(BultinRenderPassType::Ui, cmd_buffer, swap_img_idx);
        self.end_renderpass(cmd_buffer);

        unsafe { device.end_command_buffer(cmd_buffer)?; }

        Ok(())
    }

    fn record_scene_cmd_buffer(&self, device: &ash::Device, cmd_buffer: vk::CommandBuffer, render_data: &RenderData, resources: &ResourceManager) -> HellResult<()> {
        unsafe {
            let mut curr_mat_handle = ResourceHandle::MAX;
            let mut curr_mesh_idx = usize::MAX;

            let mut curr_mat = resources.material_at(0).unwrap();
            let mut curr_shader = self.shaders.get(&curr_mat.shader).unwrap();
            let mut curr_mesh = &self.meshes[0];

            let mut curr_shader_key: &str = "";


            // bind static descriptor sets once
            let descriptor_set = [
                curr_shader.get_global_set(0, self.frame_idx)?,
                curr_shader.get_object_set(0, self.frame_idx)?,
            ];

            let min_ubo_alignment = self.ctx.phys_device.device_props.limits.min_uniform_buffer_offset_alignment;
            let dynamic_descriptor_offsets = [
                SceneData::padded_device_size(min_ubo_alignment) as u32 * self.frame_idx as u32,
            ];

            device.cmd_bind_descriptor_sets(cmd_buffer, vk::PipelineBindPoint::GRAPHICS, curr_shader.pipeline.layout, 0, &descriptor_set, &dynamic_descriptor_offsets);

            // draw each object
            for (idx, rd) in render_data.iter().enumerate() {
                if curr_mat_handle != rd.material {
                    curr_mat_handle = rd.material;
                    // curr_mat = &self.materials[curr_mat];
                    curr_mat = resources.material_at(curr_mat_handle.id).to_hell_err(HellErrorKind::RenderError)?;

                    // bind material descriptors
                    let descriptor_set = [ curr_shader.get_material_set(rd.material.id)? ];
                    device.cmd_bind_descriptor_sets(cmd_buffer, vk::PipelineBindPoint::GRAPHICS, curr_shader.pipeline.layout, 2, &descriptor_set, &[]);
                }

                // bind pipeline
                if curr_shader_key != curr_mat.shader {
                    curr_shader_key = &curr_mat.shader;
                    // curr_pipeline = &self.pipelines[curr_pipeline_idx];
                    curr_shader = self.shaders.get(curr_shader_key).unwrap();
                    device.cmd_bind_pipeline(cmd_buffer, vk::PipelineBindPoint::GRAPHICS, curr_shader.pipeline.pipeline);
                }

                // bind mesh
                if curr_mesh_idx != rd.mesh_idx {
                    curr_mesh_idx = rd.mesh_idx;
                    curr_mesh = &self.meshes[curr_mesh_idx];

                    let vertex_buffers = [curr_mesh.vertex_buffer.buffer];
                    device.cmd_bind_vertex_buffers(cmd_buffer, 0, &vertex_buffers, &[0]);
                    device.cmd_bind_index_buffer(cmd_buffer, curr_mesh.index_buffer.buffer, 0, VulkanMesh::INDEX_TYPE);
                }

                // bind push constants
                let push_constants = [
                    MeshPushConstants {
                        model: rd.transform.create_model_mat()
                    }
                ];

                let push_const_bytes = std::slice::from_raw_parts(push_constants.as_ptr() as *const u8, std::mem::size_of_val(&push_constants));
                device.cmd_push_constants(cmd_buffer, curr_shader.pipeline.layout, vk::ShaderStageFlags::VERTEX, 0, push_const_bytes);

                // draw
                // value of 'first_instance' is used in the vertex shader to index into the object storage
                device.cmd_draw_indexed(cmd_buffer, curr_mesh.indices_count() as u32, 1, 0, 0, idx as u32);
            }
        }

        Ok(())
    }
}

impl VulkanBackend {
    pub fn update_global_state(&mut self, camera: TmpCamera) -> HellResult<()> {
        let global_uo = GlobalUniformObject::new(camera.view, camera.proj, camera.view_proj);

        for (_, sh) in &mut self.shaders {
            sh.update_global_uo(global_uo.clone(), &self.ctx, self.frame_idx)?;
        }

        Ok(())
    }

    pub fn update_scene_buffer(&self, scene_data: &SceneData) -> HellResult<()> {
        let min_ubo_alignment = self.ctx.phys_device.device_props.limits.min_uniform_buffer_offset_alignment;
        for (_, sh) in &self.shaders {
            let buffer = sh.get_scene_buffer();
            buffer.upload_data_buffer_array(&self.ctx.device.device, min_ubo_alignment, scene_data, self.frame_idx)?;
        }

        Ok(())
    }

    pub fn update_object_buffer(&self, render_data: &RenderData) -> HellResult<()> {
        for (_, sh) in &self.shaders {
            let buffer = sh.get_object_buffer(self.frame_idx);

            let object_data: Vec<_> = render_data.iter()
                .map(|r| ObjectData {
                    model: r.transform.create_model_mat()
                })
                .collect();

            unsafe {
                // TODO: try to write diretly into the buffer
                buffer.upload_data_storage_buffer(&self.ctx.device.device, object_data.as_ptr(), object_data.len())?;
            }
        }

        Ok(())
    }

}
