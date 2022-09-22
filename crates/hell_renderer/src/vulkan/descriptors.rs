use ash::prelude::VkResult;
use ash::vk;
use crate::vulkan::camera::VulkanCamera;

use super::image::TextureImage;
use super::{config, VulkanBuffer, VulkanSampler};










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

    pub fn new_per_frame_set_group(device: &ash::Device) -> VkResult<Self> {
        let layout = Self::create_per_frame_set_layout(device)?;

        Ok(Self::new(layout))
    }

    pub fn new_per_material_set_group(device: &ash::Device) -> VkResult<Self> {
        let layout = Self::create_per_material_set_layout(device)?;

        Ok(Self::new(layout))
    }

    fn create_per_frame_set_layout(device: &ash::Device) -> VkResult<vk::DescriptorSetLayout> {
        let layout_bindings = [
            vk::DescriptorSetLayoutBinding::builder()
                .binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX)
                .build(),
            vk::DescriptorSetLayoutBinding::builder()
                .binding(1)
                .descriptor_count(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                .build()
        ];

        let layout_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&layout_bindings)
            .build();

        let layout = unsafe { device.create_descriptor_set_layout(&layout_info, None)? };

        Ok(layout)
    }

    fn create_per_material_set_layout(device: &ash::Device) -> VkResult<vk::DescriptorSetLayout> {
        let layout_bindings = [
            vk::DescriptorSetLayoutBinding::builder()
                .binding(0)
                .descriptor_count(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                .build()
        ];

        let layout_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&layout_bindings)
            .build();

        let layout = unsafe { device.create_descriptor_set_layout(&layout_info, None)? };

        Ok(layout)
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
    pub per_frame_group: VulkanDescriptorSetGroup,
    pub per_material_group: VulkanDescriptorSetGroup,
    layouts: [vk::DescriptorSetLayout; 2],
}

impl VulkanDescriptorManager {
    pub fn new(device: &ash::Device) -> VkResult<Self> {
        let pool_sizes = [
            vk::DescriptorPoolSize::builder()
                .ty(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(config::MAX_FRAMES_IN_FLIGHT)
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

        let per_frame_group = VulkanDescriptorSetGroup::new_per_frame_set_group(device)?;
        let per_material_group = VulkanDescriptorSetGroup::new_per_material_set_group(device)?;


        let layouts = [
            per_frame_group.layout,
            per_material_group.layout,
        ];

        Ok(Self {
            pool,
            per_frame_group,
            per_material_group,
            layouts,
        })
    }

}

impl VulkanDescriptorManager {
    pub fn drop_manual(&self, device: &ash::Device) {
        println!("> dropping DescriptorPool...");

        self.per_material_group.drop_manual(device);
        self.per_frame_group.drop_manual(device);

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
    pub fn get_per_frame_set(&self, set_idx: usize, frame_idx: usize) -> vk::DescriptorSet {
        *self.per_frame_group.sets
            .get(set_idx).unwrap()
            .get(frame_idx).unwrap()
    }

    // TODO: error handling
    pub fn get_per_material_set(&self, set_idx: usize) -> vk::DescriptorSet {
        *self.per_material_group.sets
            .get(set_idx).unwrap()
            .get(0).unwrap()
    }
}

impl VulkanDescriptorManager {
    pub fn add_per_frame_descriptor_sets(
        &mut self,
        device: &ash::Device,
        uniform_buffers: &[VulkanBuffer],
        texture: &TextureImage,
        sampler: &VulkanSampler,
        descriptor_count: usize
    ) -> VkResult<usize> {
        let group = &mut self.per_frame_group;
        let layouts = vec![group.layout; descriptor_count];

        let alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(self.pool)
            .set_layouts(&layouts)
            .build();

        let sets = unsafe { device.allocate_descriptor_sets(&alloc_info)? };

        for (idx, s) in sets.iter().enumerate() {
            let buffer_infos = [
                vk::DescriptorBufferInfo::builder()
                    .buffer(uniform_buffers[idx].buffer)
                    .offset(0)
                    .range(std::mem::size_of::<VulkanCamera>() as vk::DeviceSize)
                    .build()
            ];

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
                    .dst_binding(0) // corresponds to shader binding
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .buffer_info(&buffer_infos)
                    .build(),
                vk::WriteDescriptorSet::builder()
                    .dst_set(*s)
                    .dst_binding(1)
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

    pub fn add_per_material_descriptor_sets(&mut self, device: &ash::Device, texture: &TextureImage, sampler: &VulkanSampler) -> VkResult<usize> {
        // one set for all frames
        let group = &mut self.per_material_group;
        let layouts = vec![group.layout; 1];

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