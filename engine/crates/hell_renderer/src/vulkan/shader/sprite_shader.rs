use std::array;

use ash::vk;
use hell_error::HellResult;
use crate::error::{err_invalid_frame_idx, err_invalid_set_idx};

use crate::shared::shader::sprite_shader::{SpriteShaderGlobalUniformObject, SpriteShaderSceneData, SpriteShaderObjectData};
use crate::vulkan::pipeline::VulkanPipeline;
use crate::vulkan::pipeline::shader_data::calculate_aligned_size;
use crate::vulkan::primitives::{VulkanImage, VulkanBuffer, VulkanSampler, VulkanSwapchain, VulkanDescriptorSet, VulkanRenderPassData};
use crate::vulkan::VulkanContextRef;
use crate::{vulkan::{pipeline::VulkanShader, VulkanContext}, shared::collections::PerFrame};
use hell_core::config;




const SPRITE_SHADER_DESCRIPTOR_SET_COUNT: usize = 3;
pub struct VulkanSpriteShader {
    ctx: VulkanContextRef,

    // data
    pub global_uo: SpriteShaderGlobalUniformObject,
    pub global_ubos: PerFrame<VulkanBuffer>,
    pub scene_ubo: VulkanBuffer, // one ubo for all frames
    pub object_ubos: PerFrame<VulkanBuffer>,

    pub textures: Vec<VulkanImage>,
    pub sampler: VulkanSampler,

    // descriptor sets
    pub desc_set_pool: vk::DescriptorPool,
    pub global_desc_group: VulkanDescriptorSet,
    pub object_desc_group: VulkanDescriptorSet,
    pub material_desc_group: VulkanDescriptorSet,
    desc_layouts: [vk::DescriptorSetLayout; SPRITE_SHADER_DESCRIPTOR_SET_COUNT],

    // pipeline
    pub pipeline: VulkanPipeline,

}

impl Drop for VulkanSpriteShader {
    fn drop(&mut self) {
        unsafe {
            let device = &self.ctx.device.handle;
            // automatically cleans up all associated sets
            device.destroy_descriptor_pool(self.desc_set_pool, None);
        }
    }
}

impl VulkanSpriteShader {
    pub fn new(ctx: &VulkanContextRef, swapchain: &VulkanSwapchain, shader_path: &str, render_pass_data: &VulkanRenderPassData, textures: Vec<VulkanImage>) -> HellResult<Self> {
        let device = &ctx.device.handle;

        // global uniform
        // --------------
        let global_uo = SpriteShaderGlobalUniformObject::default();
        let global_ubos = array::from_fn(|_| VulkanBuffer::from_uniform(ctx, SpriteShaderGlobalUniformObject::device_size()));

        // scene uniform
        // --------------
        let scene_ubo_size = SpriteShaderSceneData::total_size(ctx.phys_device.device_props.limits.min_uniform_buffer_offset_alignment, config::FRAMES_IN_FLIGHT as u64);
        let scene_ubo = VulkanBuffer::from_uniform(ctx, scene_ubo_size);

        // object uniform
        // --------------
        let object_ubos = array::from_fn(|_| VulkanBuffer::from_storage(ctx, SpriteShaderObjectData::total_size()));

        // texture data
        // ------------
        let sampler = VulkanSampler::new(ctx)?;

        // descriptor pool
        // ---------------
        let pool_sizes = [
            // Global
            vk::DescriptorPoolSize::builder()
                .ty(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(config::FRAMES_IN_FLIGHT as u32)
                .build(),
            // Object
            vk::DescriptorPoolSize::builder()
                .ty(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
                // when using arrays of data -> defines number of elements
                .descriptor_count(config::FRAMES_IN_FLIGHT as u32)
                .build(),
            // Object
            vk::DescriptorPoolSize::builder()
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(config::FRAMES_IN_FLIGHT as u32)
                .build(),
            // Texture
            vk::DescriptorPoolSize::builder()
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(config::FRAMES_IN_FLIGHT as u32)
                .build()
        ];

        let pool_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&pool_sizes)
            // maximum number of descriptor sets that may be allocated
            .max_sets(config::MAX_DESCRIPTOR_SET_COUNT)
            .build();

        let desc_set_pool = unsafe{ device.create_descriptor_pool(&pool_info, None)? };

        // descirptor set groups
        // ---------------------
        let mut global_desc_group = new_global_group(ctx, device, 1)?;
        let _ = Self::create_global_descriptor_sets(ctx, desc_set_pool, &mut global_desc_group, &global_ubos, &scene_ubo)?;

        let mut object_desc_group = new_object_group(ctx, device, 1)?;
        let _ = Self::add_object_descriptor_set(ctx, desc_set_pool, &mut object_desc_group, &object_ubos)?;

        let mut material_desc_group = new_material_group(ctx, device, textures.len())?;
        for tex in &textures {
            let _ = Self::add_material_descriptor_sets(ctx, desc_set_pool, &mut material_desc_group, tex, &sampler)?;
        }

        let desc_layouts = [
            global_desc_group.layout,
            object_desc_group.layout,
            material_desc_group.layout,
        ];

        // pipeline
        // --------
        let shader = VulkanShader::from_file(ctx, shader_path)?;
        let pipeline = VulkanPipeline::new(ctx, swapchain, shader, render_pass_data, &desc_layouts, true, false)?;

        Ok(Self {
            ctx: ctx.clone(),

            // shader,
            pipeline,

            global_uo,
            global_ubos,
            scene_ubo,
            object_ubos,

            textures,
            sampler,

            desc_set_pool,
            global_desc_group,
            object_desc_group,
            material_desc_group,
            desc_layouts,
        })
    }

    pub fn update_global_uo(&mut self, global_uo: SpriteShaderGlobalUniformObject, core: &VulkanContext, frame_idx: usize) -> HellResult<()> {
        self.global_uo = global_uo;

        let buffer = &self.global_ubos[frame_idx];
        buffer.upload_data_buffer(&core.device.handle, &self.global_uo)?;

        Ok(())
    }
}

impl VulkanSpriteShader {
    pub fn get_layouts(&self) -> &[vk::DescriptorSetLayout] {
        &self.desc_layouts
    }

    pub fn get_global_set(&self, set_idx: usize, frame_idx: usize) -> HellResult<vk::DescriptorSet> {
        Ok(
            *self.global_desc_group.handles
                .get(set_idx).ok_or_else(|| err_invalid_set_idx(frame_idx))?
                .get(frame_idx).ok_or_else(|| err_invalid_frame_idx(frame_idx))?
        )
    }

    pub fn get_object_set(&self, set_idx: usize, frame_idx: usize) -> HellResult<vk::DescriptorSet> {
        Ok(
            *self.object_desc_group.handles
                .get(set_idx).ok_or_else(|| err_invalid_set_idx(frame_idx))?
                .get(frame_idx).ok_or_else(|| err_invalid_frame_idx(frame_idx))?
        )
    }

    pub fn get_material_set(&self, set_idx: usize, frame_idx: usize) -> HellResult<vk::DescriptorSet> {
        Ok(
            *self.material_desc_group.handles
                .get(set_idx).ok_or_else(|| err_invalid_set_idx(set_idx))?
                .get(frame_idx).ok_or_else(|| err_invalid_frame_idx(0))?
        )
    }
}

impl VulkanSpriteShader {
    pub fn create_global_descriptor_sets(ctx: &VulkanContextRef, pool: vk::DescriptorPool, group: &mut VulkanDescriptorSet, camera_ubos: &[VulkanBuffer], scene_ubo: &VulkanBuffer) -> HellResult<usize> {
        let sets = VulkanDescriptorSet::allocate_sets_for_layout(ctx, group.layout, pool)?;

        // write sets
        // ----------
        for (idx, s) in sets.iter().enumerate() {
            let camera_buffer_infos = [
                vk::DescriptorBufferInfo::builder()
                    .buffer(camera_ubos[idx].buffer)
                    .offset(0)
                    .range(SpriteShaderGlobalUniformObject::device_size())
                    .build()
            ];

            // one buffer contains one set of data per frame -> use offset to index correct buffer
            let scene_buffer_infos = [
                vk::DescriptorBufferInfo::builder()
                    .buffer(scene_ubo.buffer)
                    .offset(0)
                    // .offset(SceneData::padded_device_size(min_ubo_alignment) * idx as u64) // hard coded offset -> for non-dynamic buffer
                    .range(SpriteShaderSceneData::device_size())
                    .build()
            ];

            let write_descriptors = [
                vk::WriteDescriptorSet::builder()
                    .dst_set(*s)
                    .dst_binding(0) // corresponds to shader binding
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .buffer_info(&camera_buffer_infos)
                    .build(),
                vk::WriteDescriptorSet::builder()
                    .dst_set(*s)
                    .dst_binding(1)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
                    .buffer_info(&scene_buffer_infos)
                    .build()
            ];

            unsafe { ctx.device.handle.update_descriptor_sets(&write_descriptors, &[]); }
        }

        // convert Vec to PerFrame and push
        let sets: PerFrame<vk::DescriptorSet> = array::from_fn(|idx| sets[idx]);
        group.handles.push(sets);
        Ok(group.handles.len() - 1)
    }

    pub fn add_object_descriptor_set(ctx: &VulkanContextRef, pool: vk::DescriptorPool, group: &mut VulkanDescriptorSet,  object_ubos: &[VulkanBuffer]) -> HellResult<usize> {
        let sets = VulkanDescriptorSet::allocate_sets_for_layout(ctx, group.layout, pool)?;

        for (idx, s) in sets.iter().enumerate() {
            let object_infos = [
                vk::DescriptorBufferInfo::builder()
                    .buffer(object_ubos[idx].buffer)
                    .offset(0)
                    .range(SpriteShaderObjectData::total_size())
                    .build()
            ];

            let write_descriptors = [
                vk::WriteDescriptorSet::builder()
                    .dst_set(*s)
                    .dst_binding(0)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                    .buffer_info(&object_infos)
                    .build()
            ];

            unsafe { ctx.device.handle.update_descriptor_sets(&write_descriptors, &[]); }
        }

        let sets: PerFrame<vk::DescriptorSet> = array::from_fn(|idx| sets[idx]);
        group.handles.push(sets);
        Ok(group.handles.len() - 1)
    }

    pub fn add_material_descriptor_sets(ctx: &VulkanContextRef, pool: vk::DescriptorPool, group: &mut VulkanDescriptorSet, texture: &VulkanImage, sampler: &VulkanSampler) -> HellResult<usize> {
        // TODO: check - can we use one set for all frames?
        let sets = VulkanDescriptorSet::allocate_sets_for_layout(ctx, group.layout, pool)?;

        for (_, s) in sets.iter().enumerate() {
            let image_infos = [
                vk::DescriptorImageInfo::builder()
                    .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .image_view(texture.view)
                    .sampler(sampler.sampler)
                    .build()
            ];

            let write_descriptors = [
                vk::WriteDescriptorSet::builder()
                    .dst_set(*s)
                    .dst_binding(0)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(&image_infos)
                    .build()
            ];

            unsafe { ctx.device.handle.update_descriptor_sets(&write_descriptors, &[]); }
        }

        let sets: PerFrame<vk::DescriptorSet> = array::from_fn(|idx| sets[idx]);
        group.handles.push(sets);

        Ok(group.handles.len() - 1)
    }
}

impl VulkanSpriteShader {
    pub fn get_scene_buffer(&self) -> &VulkanBuffer {
        &self.scene_ubo
    }

    pub fn get_all_object_buffers(&self) -> &[VulkanBuffer] {
        &self.object_ubos
    }

    pub fn get_object_buffer(&self, frame_idx: usize) -> &VulkanBuffer {
        &self.object_ubos[frame_idx]
    }
}


// ----------------------------------------------------------------------------
// ubos
// ----------------------------------------------------------------------------

pub trait VulkanUboData {
    fn device_size() -> vk::DeviceSize;

    fn padded_device_size(min_ubo_alignment: u64) -> vk::DeviceSize {
        calculate_aligned_size(min_ubo_alignment, Self::device_size())
    }
}

// ----------------------------------------------

impl VulkanUboData for SpriteShaderGlobalUniformObject {
    fn device_size() -> vk::DeviceSize {
        std::mem::size_of::<Self>() as vk::DeviceSize
    }
}

// ----------------------------------------------

impl VulkanUboData for SpriteShaderSceneData {
    fn device_size() -> vk::DeviceSize {
        std::mem::size_of::<Self>() as vk::DeviceSize
    }
}

impl SpriteShaderSceneData {
    pub fn total_size(min_ubo_alignment: u64, frame_count: u64) -> vk::DeviceSize {
        Self::padded_device_size(min_ubo_alignment) * frame_count
    }
}

// ----------------------------------------------

impl VulkanUboData for SpriteShaderObjectData {
    fn device_size() -> vk::DeviceSize {
        std::mem::size_of::<Self>() as vk::DeviceSize
    }
}

impl SpriteShaderObjectData {
    pub fn total_size() -> vk::DeviceSize {
        (Self::device_size() *  Self::MAX_OBJ_COUNT) as vk::DeviceSize
    }
}


// ----------------------------------------------------------------------------
// descriptor sets
// ----------------------------------------------------------------------------

fn new_global_group(ctx: &VulkanContextRef, device: &ash::Device, capacity: usize) -> HellResult<VulkanDescriptorSet> {
    let bindings = [
        // Global-Uniform
        vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            // number of elements in array
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .build(),
        // Scene-Data
        vk::DescriptorSetLayoutBinding::builder()
            .binding(1)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
            .build()
    ];

    let layout = VulkanDescriptorSet::create_descriptor_set_layout(device, &bindings)?;

    Ok(VulkanDescriptorSet::new(ctx, layout, capacity))
}

fn new_object_group(ctx: &VulkanContextRef, device: &ash::Device, capacity: usize) -> HellResult<VulkanDescriptorSet> {
    let bindings = [
        // Per-Object-Data
        vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .build()
    ];

    let layout = VulkanDescriptorSet::create_descriptor_set_layout(device, &bindings)?;

    Ok(VulkanDescriptorSet::new(ctx, layout, capacity))
}

fn new_material_group(ctx: &VulkanContextRef, device: &ash::Device, capacity: usize) -> HellResult<VulkanDescriptorSet> {
    let bindings = [
        // texture_sampler
        vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .build()
    ];

    let layout = VulkanDescriptorSet::create_descriptor_set_layout(device, &bindings)?;

    Ok(VulkanDescriptorSet::new(ctx, layout, capacity))
}
