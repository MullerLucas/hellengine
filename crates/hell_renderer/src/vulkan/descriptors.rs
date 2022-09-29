use ash::prelude::VkResult;
use ash::vk;


use crate::shared::render_data::{CameraData, SceneData, ObjectData};

use super::image::TextureImage;
use super::{config, VulkanBuffer, VulkanSampler, VulkanUboData};








// ----------------------------------------------------------------------------
// descriptor-pool
// ----------------------------------------------------------------------------

pub struct VulkanDescriptorSetGroup {
    pub layout: vk::DescriptorSetLayout,
    pub sets: Vec<Vec<vk::DescriptorSet>>, // per frame
}

impl VulkanDescriptorSetGroup {
    pub fn new(layout: vk::DescriptorSetLayout) -> Self {
        Self {
            layout,
            sets: vec![]
        }
    }

    pub fn new_global_group(device: &ash::Device) -> VkResult<Self> {
        let layout = Self::create_global_set_layout(device)?;

        Ok(Self::new(layout))
    }

    pub fn new_object_group(device: &ash::Device) -> VkResult<Self> {
        let layout = Self::create_object_set_layout(device)?;

        Ok(Self::new(layout))
    }

    pub fn new_material_group(device: &ash::Device) -> VkResult<Self> {
        let layout = Self::create_material_set_layout(device)?;

        Ok(Self::new(layout))
    }

    fn create_global_set_layout(device: &ash::Device) -> VkResult<vk::DescriptorSetLayout> {
        let bindings = [
            vk::DescriptorSetLayoutBinding::builder()
                .binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX)
                .build(),
            vk::DescriptorSetLayoutBinding::builder()
                .binding(1)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
                .build()
        ];

        Self::create_descriptor_set_layout(device, &bindings)
    }

    fn create_object_set_layout(device: &ash::Device) -> VkResult<vk::DescriptorSetLayout> {
        let bindings = [
            vk::DescriptorSetLayoutBinding::builder()
                .binding(0)
                .descriptor_count(1)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .stage_flags(vk::ShaderStageFlags::VERTEX)
                .build()
        ];

        Self::create_descriptor_set_layout(device, &bindings)
    }

    fn create_material_set_layout(device: &ash::Device) -> VkResult<vk::DescriptorSetLayout> {
        let bindings = [
            vk::DescriptorSetLayoutBinding::builder()
                .binding(0)
                .descriptor_count(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                .build()
        ];

        Self::create_descriptor_set_layout(device, &bindings)
    }

    fn create_descriptor_set_layout(device: &ash::Device, bindings: &[vk::DescriptorSetLayoutBinding]) -> VkResult<vk::DescriptorSetLayout> {
        let layout_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(bindings)
            .build();

        Ok(unsafe {
            device.create_descriptor_set_layout(&layout_info, None)?
        })
    }
}

impl VulkanDescriptorSetGroup {
    pub fn drop_manual(&self, device: &ash::Device) {
        println!("> dropping VulkanDescriptorSetLayoutGroup...");

        unsafe {
            device.destroy_descriptor_set_layout(self.layout, None);
        }
    }
}

impl VulkanDescriptorSetGroup {
    pub fn set_count(&self) -> usize {
        self.sets.len()
    }
}





// ----------------------------------------------------------------------------
// descriptor
// ----------------------------------------------------------------------------

pub struct VulkanDescriptorManager {
    pool: vk::DescriptorPool,
    pub global_group: VulkanDescriptorSetGroup,
    pub object_group: VulkanDescriptorSetGroup,
    pub material_group: VulkanDescriptorSetGroup,
    layouts: [vk::DescriptorSetLayout; 3],
}

impl VulkanDescriptorManager {
    pub fn new(device: &ash::Device) -> VkResult<Self> {
        let pool_sizes = [
            vk::DescriptorPoolSize::builder()
                .ty(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(config::MAX_FRAMES_IN_FLIGHT as u32)
                .build(),
            vk::DescriptorPoolSize::builder()
                .ty(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
                .descriptor_count(config::DYNAMIC_UNIFORM_DESCRIPTOR_COUNT)
                .build(),
            vk::DescriptorPoolSize::builder()
                .ty(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(config::DYNAMIC_STORAGE_DESCRIPTOR_COUNT)
                .build(),
            vk::DescriptorPoolSize::builder()
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(config::TEXTURE_DESCRIPTOR_COUNT)
                .build()
        ];

        let pool_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&pool_sizes)
            .max_sets(config::MAX_DESCRIPTOR_SET_COUNT)
            .build();

        let pool = unsafe{ device.create_descriptor_pool(&pool_info, None)? };

        let global_group = VulkanDescriptorSetGroup::new_global_group(device)?;
        let object_group = VulkanDescriptorSetGroup::new_object_group(device)?;
        let material_group = VulkanDescriptorSetGroup::new_material_group(device)?;

        let layouts = [
            global_group.layout,
            object_group.layout,
            material_group.layout,
        ];

        Ok(Self {
            pool,
            global_group,
            object_group,
            material_group,
            layouts,
        })
    }

}

impl VulkanDescriptorManager {
    pub fn drop_manual(&self, device: &ash::Device) {
        println!("> dropping DescriptorPool...");

        self.material_group.drop_manual(device);
        self.object_group.drop_manual(device);
        self.global_group.drop_manual(device);

        unsafe {
            // automatically cleans up all associated sets
            device.destroy_descriptor_pool(self.pool, None);
        }
    }
}

impl VulkanDescriptorManager {
    pub fn get_layouts(&self) -> &[vk::DescriptorSetLayout] {
        &self.layouts
    }

    // TODO: error handling
    pub fn get_global_set(&self, set_idx: usize, frame_idx: usize) -> vk::DescriptorSet {
        *self.global_group.sets
            .get(set_idx).unwrap()
            .get(frame_idx).unwrap()
    }

    // TODO: error handling
    pub fn get_object_set(&self, set_idx: usize, frame_idx: usize) -> vk::DescriptorSet {
        *self.object_group.sets
            .get(set_idx).unwrap()
            .get(frame_idx).unwrap()
    }

    // TODO: error handling
    pub fn get_material_set(&self, set_idx: usize) -> vk::DescriptorSet {
        *self.material_group.sets
            .get(set_idx).unwrap()
            .get(0).unwrap()
    }
}

impl VulkanDescriptorManager {
    pub fn add_global_descriptor_sets(&mut self, device: &ash::Device, camera_ubos: &[VulkanBuffer], scene_ubo: &VulkanBuffer, descriptor_count: usize) -> VkResult<usize> {
        let group = &mut self.global_group;
        let layouts = vec![group.layout; descriptor_count];

        // create sets
        // -----------
        let alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.pool)
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
                    .range(CameraData::device_size())
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

    pub fn add_object_descriptor_set(&mut self, device: &ash::Device, object_ubos: &[VulkanBuffer], descriptor_count: usize) -> VkResult<usize> {
        let group = &mut self.object_group;
        let layouts = vec![group.layout; descriptor_count];

        let alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.pool)
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

    pub fn add_material_descriptor_sets(&mut self, device: &ash::Device, texture: &TextureImage, sampler: &VulkanSampler) -> VkResult<usize> {
        // one set for all frames
        let group = &mut self.material_group;
        let layouts = vec![group.layout];

        let alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.pool)
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
