use ash::vk;
use hell_common::transform::Transform;
use hell_common::window::HellWindowExtent;
use crate::vulkan::image::TextureImage;
use crate::vulkan::{VulkanLogicDevice, VulkanSampler};
use crate::vulkan::descriptors::VulkanDescriptorManager;

use super::buffer::VulkanBuffer;
use super::{config, SceneData, VulkanUboData};
use super::frame::VulkanFrameData;
use super::pipeline::VulkanPipeline;
use super::render_pass::VulkanRenderPassData;
use super::vertext::Vertex;
use super::vulkan_core::VulkanCore;



static QUAD_VERTS: &[Vertex] = &[
    Vertex::from_arrays([-1.0, -1.0,  0.0, 1.0], [1.0, 0.0, 0.0, 1.0], [1.0, 0.0]),
    Vertex::from_arrays([ 1.0, -1.0,  0.0, 1.0], [0.0, 1.0, 0.0, 1.0], [0.0, 0.0]),
    Vertex::from_arrays([ 1.0,  1.0,  0.0, 1.0], [0.0, 0.0, 1.0, 1.0], [0.0, 1.0]),
    Vertex::from_arrays([-1.0,  1.0,  0.0, 1.0], [1.0, 1.0, 1.0, 1.0], [1.0, 1.0]),
];

static QUAD_INDICES: &[u32] = &[
    0, 1, 2,
    2, 3, 0,
];



// ----------------------------------------------------------------------------
// mesh
// ----------------------------------------------------------------------------

pub struct VulkanMesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,

    pub vertex_buffer: VulkanBuffer,
    pub index_buffer: VulkanBuffer,
}

impl VulkanMesh {
    pub const INDEX_TYPE: vk::IndexType = vk::IndexType::UINT32;

    pub fn new_quad(core: &VulkanCore) -> Self {
        Self {
            vertices: QUAD_VERTS.to_vec(),
            indices: QUAD_INDICES.to_vec(),

            vertex_buffer: VulkanBuffer::from_vertices(core, QUAD_VERTS),
            index_buffer: VulkanBuffer::from_indices(core, QUAD_INDICES),
        }
    }

    pub fn indices_count(&self) -> usize {
        self.indices.len()
    }
}

impl VulkanMesh {
    fn drop_manual(&mut self, device: &VulkanLogicDevice) {
        self.vertex_buffer.drop_manual(&device.device);
        self.index_buffer.drop_manual(&device.device);
    }
}


// ----------------------------------------------------------------------------
// render data
// ----------------------------------------------------------------------------

pub struct VulkanMaterial {
    pub pipeline_idx: usize,
    pub texture_idx: usize,
    pub descriptor_set_idx: usize,
}

impl VulkanMaterial {
    pub fn new(pipeline_idx: usize, texture_idx: usize, descriptor_set_idx: usize) -> Self {
        Self {
            pipeline_idx,
            texture_idx,
            descriptor_set_idx,
        }
    }
}

// ----------------------------------------------------------------------------
// render data
// ----------------------------------------------------------------------------

pub struct RenderDataChunk<'a> {
    // pub pipeline_idx: usize,
    pub mesh_idx: usize,
    pub transform: &'a Transform,
    pub material_idx: usize,
}

#[derive(Default)]
pub struct RenderData {
    pub mesh_indices: Vec<usize>,
    // pub pipeline_indices: Vec<usize>,
    // pub texture_descriptor_set: Vec<vk::DescriptorSet>,
    pub material_indices: Vec<usize>,

    pub transforms: Vec<Transform>,
}

impl RenderData {
    pub fn data_count(&self) -> usize {
        self.mesh_indices.len()
    }

    pub fn add_data(&mut self, mesh_idx: usize, material_idx: usize, trans: Transform) -> usize {
        self.mesh_indices.push(mesh_idx);
        self.transforms.push(trans);
        self.material_indices.push(material_idx);
        // self.pipeline_indices.push(pipeline_idx);
        // self.texture_descriptor_set.push(texture_descriptor);

        self.data_count()
    }

    pub fn data_at(&self, idx: usize) -> RenderDataChunk {
        RenderDataChunk {
            mesh_idx: self.mesh_indices[idx],
            transform: &self.transforms[idx],
            material_idx: self.material_indices[idx]
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
// push-constants
// ----------------------------------------------------------------------------

#[derive(Debug)]
pub struct MeshPushConstants {
    pub model: glam::Mat4,
}

// ----------------------------------------------------------------------------
// renderer
// ----------------------------------------------------------------------------

pub struct VulkanRenderer2D {
    pub curr_frame_idx: usize,
    pub frame_data: VulkanFrameData,

    pub pipelines: Vec<VulkanPipeline>,
    pub meshes: Vec<VulkanMesh>,
    pub texture: Vec<TextureImage>,
    pub sampler: Vec<VulkanSampler>,
    pub materials: Vec<VulkanMaterial>,
    pub descriptor_manager: VulkanDescriptorManager,

    pub render_pass_data: VulkanRenderPassData,
    pub core: VulkanCore,
}

impl VulkanRenderer2D {
    // TODO: error handling
    pub fn new(core: VulkanCore) -> Self {
        let frame_data = VulkanFrameData::new(&core);

        let quad_mesh = VulkanMesh::new_quad(&core);
        let meshes = vec![quad_mesh];
        let texture = vec![
            TextureImage::new(&core, config::TEXTURE_0_PATH),
            TextureImage::new(&core, config::TEXTURE_1_PATH),
            TextureImage::new(&core, config::TEXTURE_2_PATH)
        ];
        let sampler = vec![VulkanSampler::new(&core).unwrap()];

        let device = &core.device.device;
        let mut descriptor_manager = VulkanDescriptorManager::new(device).unwrap();
        let _ = descriptor_manager.add_global_descriptor_sets(device, &frame_data.camera_ubos, &frame_data.scene_ubo, config::MAX_FRAMES_IN_FLIGHT as usize).unwrap();
        let _ = descriptor_manager.add_object_descriptor_set(device, &frame_data.object_ubos, config::MAX_FRAMES_IN_FLIGHT as usize).unwrap();
        let _ = descriptor_manager.add_material_descriptor_sets(device, &texture[0], &sampler[0]).unwrap();
        let _ = descriptor_manager.add_material_descriptor_sets(device, &texture[1], &sampler[0]).unwrap();
        let _ = descriptor_manager.add_material_descriptor_sets(device, &texture[2], &sampler[0]).unwrap();

        let render_pass_data = VulkanRenderPassData::new(&core);
        let default_pipeline = VulkanPipeline::new(&core, &render_pass_data, descriptor_manager.get_layouts());
        let pipelines = vec![default_pipeline];

        let materials = vec![
            VulkanMaterial::new(0, 0, 0),
            VulkanMaterial::new(0, 1, 1),
            VulkanMaterial::new(0, 2, 2),
        ];

        Self {
            curr_frame_idx: 0,
            frame_data,

            pipelines,
            meshes,
            texture,
            sampler,
            materials,
            descriptor_manager,

            render_pass_data,
            core,
        }
    }
}

impl Drop for VulkanRenderer2D {
    fn drop(&mut self) {
        println!("> dropping Renerer2D...");

        let device = &self.core.device.device;

        self.meshes.iter_mut().for_each(|m| m.drop_manual(&self.core.device));

        self.texture.iter().for_each(|t| t.drop_manual(device));
        self.sampler.iter().for_each(|s| s.drop_manual(device));

        self.descriptor_manager.drop_manual(device);

        self.frame_data.drop_manual(device);
        self.render_pass_data.drop_manual(&self.core.device.device);
        self.pipelines.iter_mut().for_each(|p| { p.drop_manual(&self.core.device.device) });
    }
}

impl VulkanRenderer2D {
    pub fn wait_idle(&self) {
        self.core.wait_device_idle();
    }

    pub fn on_window_changed(&mut self, window_extent: &HellWindowExtent) {
        self.core.recreate_swapchain(window_extent);
        self.render_pass_data.recreate_framebuffer(&self.core);
    }

    pub fn draw_frame(&mut self, _delta_time: f32, render_data: &RenderData) -> bool {
        let core = &self.core;
        let device = &core.device.device;
        let render_pass_data = &self.render_pass_data;

        // let frame_data = &self.frame_data[frame_idx];
        let cmd_pool = &self.frame_data.graphics_cmd_pools.get(self.curr_frame_idx).unwrap();
        self.frame_data.wait_for_in_flight(&self.core.device.device, self.curr_frame_idx);

        // TODO: check
        // let swap_img_idx = match self.swapchain.aquire_next_image(frame_data.img_available_sem[0]) {
        //     Ok((idx, _)) => { idx },
        //     Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => { return true },
        //     _ => { panic!("failed to aquire next image") }
        // };
        let (curr_swap_idx, _is_suboptimal) = core.swapchain.aquire_next_image(self.frame_data.img_available_semaphors[self.curr_frame_idx][0]).unwrap();

        cmd_pool.reset_cmd_buffer(device, 0);
        self.record_cmd_buffer(
            core,
            render_pass_data,
            curr_swap_idx as usize,
            render_data
        );

        // delay resetting the fence until we know for sure we will be submitting work with it (not return early)
        self.frame_data.reset_in_flight_fence(device, self.curr_frame_idx);
        self.frame_data.submit_queue(device, core.device.queues.graphics.queue, &[cmd_pool.get_buffer(0)], self.curr_frame_idx);

        let present_result = self.frame_data.present_queue(core.device.queues.present.queue, &core.swapchain, &[curr_swap_idx], self.curr_frame_idx);

        // TODO: check
        // do this after queue-present to ensure that the semaphores are in a consistent state - otherwise a signaled semaphore may never be properly waited upon
        let is_resized = match present_result {
            Ok(is_suboptimal) => { is_suboptimal },
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR | vk::Result::SUBOPTIMAL_KHR)  => { true },
            _ => { panic!("failed to aquire next image") }
        };

        self.curr_frame_idx = (self.curr_frame_idx + 1) % config::MAX_FRAMES_IN_FLIGHT as usize;

        is_resized
    }

    fn record_cmd_buffer(
        &self,
        core: &VulkanCore,
        render_pass_data: &VulkanRenderPassData,
        swap_img_idx: usize,
        render_data: &RenderData
    ) {
        let begin_info = vk::CommandBufferBeginInfo::default();
        let device = &core.device.device;
        let cmd_buffer = self.frame_data.get_cmd_buffer(self.curr_frame_idx);

        unsafe { device.begin_command_buffer(cmd_buffer, &begin_info).unwrap(); }

        // one clear-color per attachment with load-op-clear - order should be identical
        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue { float32: config::CLEAR_COLOR }
            },
            vk::ClearValue {
                // range of depth: [0, 1]
                depth_stencil: vk::ClearDepthStencilValue{ depth: 1.0, stencil: 0 }
            }
        ];

        let render_area = vk::Rect2D {
            offset: vk::Offset2D::default(),
            extent: core.swapchain.extent
        };

        let render_pass_info = vk::RenderPassBeginInfo::builder()
            .render_pass(render_pass_data.render_pass.render_pass)
            .framebuffer(render_pass_data.framebuffer.buffer_at(swap_img_idx))
            .clear_values(&clear_values)
            .render_area(render_area)
            .build();

        unsafe { device.cmd_begin_render_pass(cmd_buffer, &render_pass_info, vk::SubpassContents::INLINE); }

        // record commands
        self.record_scene_cmd_buffer(device, cmd_buffer, render_data);

        unsafe {
            device.cmd_end_render_pass(cmd_buffer);
        }

        unsafe {
            device.end_command_buffer(cmd_buffer).unwrap();
        }
    }

    fn record_scene_cmd_buffer(&self, device: &ash::Device, cmd_buffer: vk::CommandBuffer, render_data: &RenderData) {
        unsafe {
            let mut curr_pipeline_idx = usize::MAX;
            let mut curr_mat_idx = usize::MAX;
            let mut curr_mesh_idx = usize::MAX;

            let mut curr_mat = &self.materials[0];
            let mut curr_pipeline = &self.pipelines[0];
            let mut curr_mesh = &self.meshes[0];



            // bind static descriptor sets once
            let descriptor_set = [
                self.descriptor_manager.get_global_set(0, self.curr_frame_idx),
                self.descriptor_manager.get_object_set(0, self.curr_frame_idx),
            ];

            let min_ubo_alignment = self.core.phys_device.device_props.limits.min_uniform_buffer_offset_alignment;
            let dynamic_descriptor_offsets = [
                SceneData::padded_device_size(min_ubo_alignment) as u32 * self.curr_frame_idx as u32,
            ];

            device.cmd_bind_descriptor_sets(cmd_buffer, vk::PipelineBindPoint::GRAPHICS, curr_pipeline.pipeline_layout, 0, &descriptor_set, &dynamic_descriptor_offsets);



            // draw each object
            for (idx, rd) in render_data.iter().enumerate() {

                if curr_mat_idx != rd.material_idx {
                    curr_mat_idx = rd.material_idx;
                    curr_mat = &self.materials[curr_mat_idx];

                    // bind material descriptors
                    let descriptor_set = [ self.descriptor_manager.get_material_set(rd.material_idx) ];
                    device.cmd_bind_descriptor_sets(cmd_buffer, vk::PipelineBindPoint::GRAPHICS, curr_pipeline.pipeline_layout, 2, &descriptor_set, &[]);
                }

                // bind pipeline
                if curr_pipeline_idx != curr_mat.pipeline_idx {
                    curr_pipeline_idx = curr_mat.pipeline_idx;
                    curr_pipeline = &self.pipelines[curr_pipeline_idx];
                    device.cmd_bind_pipeline(cmd_buffer, vk::PipelineBindPoint::GRAPHICS, curr_pipeline.pipeline);
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
                device.cmd_push_constants(cmd_buffer, curr_pipeline.pipeline_layout, vk::ShaderStageFlags::VERTEX, 0, push_const_bytes);

                // draw
                // value of 'first_instance' is used in the vertex shader to index into the object storage
                device.cmd_draw_indexed(cmd_buffer, curr_mesh.indices_count() as u32, 1, 0, 0, idx as u32);
            }
        }
    }
}
