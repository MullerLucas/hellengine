use ash::prelude::VkResult;
use ash::vk;

use super::buffer::VulkanUniformBufferObject;
use super::image::VulkanTextureImage;
use super::{config, VulkanBuffer, VulkanSampler};



// ----------------------------------------------------------------------------
// descriptor-layout
// ----------------------------------------------------------------------------

pub struct VulkanDescriptorSetLayout {
    pub layout: vk::DescriptorSetLayout,
}

impl VulkanDescriptorSetLayout {
    pub fn new(device: &ash::Device) -> VkResult<Self> {
        let laytout_bindings = [
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
            .bindings(&laytout_bindings)
            .build();

        let layout = unsafe { device.create_descriptor_set_layout(&layout_info, None)? };

        Ok(Self {
            layout
        })
    }
}

impl VulkanDescriptorSetLayout {
    pub fn drop_manual(&self, device: &ash::Device) {
        println!("> dropping DescriptorLayout...");

        unsafe {
            device.destroy_descriptor_set_layout(self.layout, None);
        }
    }
}





// ----------------------------------------------------------------------------
// descriptor-pool
// ----------------------------------------------------------------------------

pub struct VulkanDescriptorPool {
    pub layout: VulkanDescriptorSetLayout,
    pub pool: vk::DescriptorPool,
    pub sets: Vec<vk::DescriptorSet>,
}

impl VulkanDescriptorPool {
    pub fn new(device: &ash::Device, uniform_buffer: &[VulkanBuffer], texture: &VulkanTextureImage, sampler: &VulkanSampler) -> VkResult<Self> {
        let layout = VulkanDescriptorSetLayout::new(device)?;

        let pool_sizes = [
            vk::DescriptorPoolSize::builder()
                .ty(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(config::MAX_FRAMES_IN_FLIGHT)
                .build(),
            vk::DescriptorPoolSize::builder()
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(config::MAX_FRAMES_IN_FLIGHT)
                .build()
        ];

        let pool_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&pool_sizes)
            .max_sets(config::MAX_FRAMES_IN_FLIGHT)
            .build();

        let pool = unsafe{ device.create_descriptor_pool(&pool_info, None)? };

        let sets = create_descriptor_sets(device, pool, layout.layout, uniform_buffer, texture, sampler)?;

        Ok(Self {
            layout,
            sets,
            pool,
        })
    }
}

impl VulkanDescriptorPool {
    pub fn drop_manual(&self, device: &ash::Device) {
        println!("> dropping DescriptorPool...");

        self.layout.drop_manual(device);

        unsafe {
            // automatically cleans up all associated sets
            device.destroy_descriptor_pool(self.pool, None);
        }
    }
}



fn create_descriptor_sets(device: &ash::Device, pool: vk::DescriptorPool, layout: vk::DescriptorSetLayout, uniform_buffers: &[VulkanBuffer], texture: &VulkanTextureImage, sampler: &VulkanSampler) -> VkResult<Vec<vk::DescriptorSet>> {
    let layouts = [layout; config::MAX_FRAMES_IN_FLIGHT as usize];

    let alloc_info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(pool)
        .set_layouts(&layouts)
        .build();

    let sets = unsafe { device.allocate_descriptor_sets(&alloc_info)? };

    for (idx, s) in sets.iter().enumerate() {
        let buffer_infos = [
            vk::DescriptorBufferInfo::builder()
                .buffer(uniform_buffers[idx].buffer)
                .offset(0)
                .range(std::mem::size_of::<VulkanUniformBufferObject>() as vk::DeviceSize)
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



    Ok(sets)
}
