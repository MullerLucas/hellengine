use ash::vk;
use hell_core::config;
use hell_error::HellResult;
use crate::error::{err_invalid_frame_idx, err_invalid_set_idx};

use crate::render_data::{SceneData, ObjectData};
use crate::vulkan::VulkanSampler;
use crate::vulkan::image::TextureImage;
use crate::{vulkan::{pipeline::{VulkanPipeline, VulkanShader, shader_data::VulkanUboData}, VulkanCore, VulkanRenderPassData, VulkanBuffer, descriptors::VulkanDescriptorSetGroup}, render_data::GlobalUniformObject, shared::collections::PerFrame};




pub struct VulkanSpriteShader {
    // data
    pub global_uo: GlobalUniformObject,
    pub global_ubos: PerFrame<VulkanBuffer>,
    pub scene_ubo: VulkanBuffer, // one ubo for all frames
    pub object_ubos: PerFrame<VulkanBuffer>,

    pub textures: Vec<TextureImage>,
    pub sampler: VulkanSampler,

    // descriptor sets
    pub desc_set_pool: vk::DescriptorPool,
    pub global_desc_group: VulkanDescriptorSetGroup,
    pub object_desc_group: VulkanDescriptorSetGroup,
    pub material_desc_group: VulkanDescriptorSetGroup,
    layouts: [vk::DescriptorSetLayout; 3],

    // pipeline
    pub shader: VulkanShader,
    pub pipeline: VulkanPipeline,

}

impl VulkanSpriteShader {
    pub fn new(core: &VulkanCore, shader_path: &str, render_pass_data: &VulkanRenderPassData, textures: Vec<TextureImage>) -> HellResult<Self> {
        let device = &core.device.device;

        // global uniform
        // --------------
        let global_uo = GlobalUniformObject::default();

        let mut global_ubo: [VulkanBuffer; config::FRAMES_IN_FLIGHT] = Default::default();
        for ubo in &mut global_ubo {
            *ubo = VulkanBuffer::from_uniform(core, GlobalUniformObject::device_size())
        }
        let global_ubos = PerFrame::new(global_ubo);

        // scene uniform
        // --------------
        let scene_ubo_size = SceneData::total_size(core.phys_device.device_props.limits.min_uniform_buffer_offset_alignment, config::FRAMES_IN_FLIGHT as u64);
        let scene_ubo = VulkanBuffer::from_uniform(core, scene_ubo_size);

        // object uniform
        // --------------
        let object_ubo_size = ObjectData::total_size();
        let mut object_ubos: [VulkanBuffer; config::FRAMES_IN_FLIGHT] = Default::default();
        for ubo in &mut object_ubos {
            *ubo = VulkanBuffer::from_storage(core, object_ubo_size);
        }
        let object_ubos = PerFrame::new(object_ubos);

        // texture data
        // ------------
        let sampler = VulkanSampler::new(&core)?;

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
                .descriptor_count(config::DYNAMIC_UNIFORM_DESCRIPTOR_COUNT)
                .build(),
            // Object
            vk::DescriptorPoolSize::builder()
                .ty(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(config::DYNAMIC_STORAGE_DESCRIPTOR_COUNT)
                .build(),
            // Texture
            vk::DescriptorPoolSize::builder()
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(config::TEXTURE_DESCRIPTOR_COUNT)
                .build()
        ];

        let pool_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&pool_sizes)
            .max_sets(config::MAX_DESCRIPTOR_SET_COUNT)
            .build();

        let desc_set_pool = unsafe{ device.create_descriptor_pool(&pool_info, None)? };

        // descirptor set groups
        // ---------------------
        let mut global_desc_group = VulkanDescriptorSetGroup::new_global_group(device)?;
        let _ = Self::add_global_descriptor_sets(device, desc_set_pool, &mut global_desc_group, global_ubos.get_all(), &scene_ubo, config::FRAMES_IN_FLIGHT)?;

        let mut object_desc_group = VulkanDescriptorSetGroup::new_object_group(device)?;
        let _ = Self::add_object_descriptor_set(device, desc_set_pool, &mut object_desc_group, object_ubos.get_all(), config::FRAMES_IN_FLIGHT)?;

        let mut material_desc_group = VulkanDescriptorSetGroup::new_material_group(device)?;
        for tex in &textures {
            let _ = Self::add_material_descriptor_sets(device, desc_set_pool, &mut material_desc_group, tex, &sampler)?;
        }

        let desc_layouts = [
            global_desc_group.layout,
            object_desc_group.layout,
            material_desc_group.layout,
        ];


        // pipeline
        // --------
        let shader = VulkanShader::from_file(
            &core.device.device,
            shader_path,
        )?;

        let pipeline = VulkanPipeline::new(core, &shader, render_pass_data, &desc_layouts)?;


        Ok(Self {
            shader,
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
            layouts: desc_layouts,
        })
    }

    pub fn drop_manual(&self, device: &ash::Device) {
        // descriptor sets
        // ---------------
        self.material_desc_group.drop_manual(device);
        self.object_desc_group.drop_manual(device);
        self.global_desc_group.drop_manual(device);

        unsafe {
            // automatically cleans up all associated sets
            device.destroy_descriptor_pool(self.desc_set_pool, None);
        }


        self.textures.iter().for_each(|t| t.drop_manual(device));
        self.sampler.drop_manual(device);

        // ubos
        // ----
        self.object_ubos.get_all().iter().for_each(|p| p.drop_manual(device));
        self.scene_ubo.drop_manual(device);
        self.global_ubos.get_all().iter() .for_each(|u| u.drop_manual(device));
    }

    pub fn update_global_uo(&mut self, global_uo: GlobalUniformObject, core: &VulkanCore, frame_idx: usize) -> HellResult<()> {
        self.global_uo = global_uo;
        let buffer = self.global_ubos.get(frame_idx);
        buffer.upload_data_buffer(&core.device.device, &self.global_uo)?;

        Ok(())
    }

}

impl VulkanSpriteShader {
    pub fn get_layouts(&self) -> &[vk::DescriptorSetLayout] {
        &self.layouts
    }

    pub fn get_global_set(&self, set_idx: usize, frame_idx: usize) -> HellResult<vk::DescriptorSet> {
        Ok(
            *self.global_desc_group.sets
                .get(set_idx).ok_or_else(|| err_invalid_set_idx(frame_idx))?
                .get(frame_idx).ok_or_else(|| err_invalid_frame_idx(frame_idx))?
        )
    }

    pub fn get_object_set(&self, set_idx: usize, frame_idx: usize) -> HellResult<vk::DescriptorSet> {
        Ok(
            *self.object_desc_group.sets
                .get(set_idx).ok_or_else(|| err_invalid_set_idx(frame_idx))?
                .get(frame_idx).ok_or_else(|| err_invalid_frame_idx(frame_idx))?
        )
    }

    pub fn get_material_set(&self, set_idx: usize) -> HellResult<vk::DescriptorSet> {
        Ok(
            *self.material_desc_group.sets
                .get(set_idx).ok_or_else(|| err_invalid_set_idx(set_idx))?
                .get(0).ok_or_else(|| err_invalid_frame_idx(0))?
        )
    }
}


impl VulkanSpriteShader {
    pub fn add_global_descriptor_sets(device: &ash::Device, pool: vk::DescriptorPool, group: &mut VulkanDescriptorSetGroup, camera_ubos: &[VulkanBuffer], scene_ubo: &VulkanBuffer, frames_in_flight: usize) -> HellResult<usize> {
        let layouts = vec![group.layout; frames_in_flight];

        // create sets
        // -----------
        let alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(pool)
            .set_layouts(&layouts)
            .build();

        let sets = unsafe { device.allocate_descriptor_sets(&alloc_info)? };

        // write sets
        // ----------
        for (idx, s) in sets.iter().enumerate() {
            let camera_buffer_infos = [
                vk::DescriptorBufferInfo::builder()
                    .buffer(camera_ubos[idx].buffer)
                    .offset(0)
                    .range(GlobalUniformObject::device_size())
                    .build()
            ];

            // one buffer contains one set of data per frame -> use offset to index correct buffer
            let scene_buffer_infos = [
                vk::DescriptorBufferInfo::builder()
                    .buffer(scene_ubo.buffer)
                    .offset(0)
                    // .offset(SceneData::padded_device_size(min_ubo_alignment) * idx as u64) // hard coded offset -> for non-dynamic buffer
                    .range(SceneData::device_size())
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

            unsafe {
                device.update_descriptor_sets(&write_descriptors, &[]);
            }
        }

        group.sets.push(sets);

        let set_idx = group.set_count() - 1;
        Ok(set_idx)
    }

    pub fn add_object_descriptor_set(device: &ash::Device, pool: vk::DescriptorPool, group: &mut VulkanDescriptorSetGroup,  object_ubos: &[VulkanBuffer], descriptor_count: usize) -> HellResult<usize> {
        let layouts = vec![group.layout; descriptor_count];

        let alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(pool)
            .set_layouts(&layouts)
            .build();

        let sets = unsafe { device.allocate_descriptor_sets(&alloc_info)? };

        for (idx, s) in sets.iter().enumerate() {
            let object_infos = [
                vk::DescriptorBufferInfo::builder()
                    .buffer(object_ubos[idx].buffer)
                    .offset(0)
                    .range(ObjectData::total_size())
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

            unsafe {
                device.update_descriptor_sets(&write_descriptors, &[]);
            }
        }

        group.sets.push(sets);
        let set_idx = group.set_count() - 1;

        Ok(set_idx)
    }

    pub fn add_material_descriptor_sets(device: &ash::Device, pool: vk::DescriptorPool, group: &mut VulkanDescriptorSetGroup, texture: &TextureImage, sampler: &VulkanSampler) -> HellResult<usize> {
        // one set for all frames
        let layouts = vec![group.layout];

        let alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(pool)
            .set_layouts(&layouts)
            .build();

        let sets = unsafe { device.allocate_descriptor_sets(&alloc_info)? };

        for (_, s) in sets.iter().enumerate() {
            let image_infos = [
                vk::DescriptorImageInfo::builder()
                    .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .image_view(texture.img.view)
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

            unsafe {
                device.update_descriptor_sets(&write_descriptors, &[]);
            }
        }

        group.sets.push(sets);

        let set_idx = group.set_count() - 1;
        Ok(set_idx)
    }
}

impl VulkanSpriteShader {
    pub fn get_scene_buffer(&self) -> &VulkanBuffer {
        &self.scene_ubo
    }

    pub fn get_all_object_buffers(&self) -> &[VulkanBuffer] {
        self.object_ubos.get_all()
    }

    pub fn get_object_buffer(&self, frame_idx: usize) -> &VulkanBuffer {
        self.object_ubos.get(frame_idx)
    }
}
