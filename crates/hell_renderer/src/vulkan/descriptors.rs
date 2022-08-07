use ash::prelude::VkResult;
use ash::vk;

use super::buffer::UniformBufferObject;
use super::{config, Buffer};



// ----------------------------------------------------------------------------
// descriptor-layout
// ----------------------------------------------------------------------------

pub struct DescriptorSetLayout {
    pub layout: vk::DescriptorSetLayout,
}

impl DescriptorSetLayout {
    pub fn new(device: &ash::Device) -> VkResult<Self> {
        let laytout_binding = vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .build();

        let layout_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&[laytout_binding])
            .build();

        let layout = unsafe { device.create_descriptor_set_layout(&layout_info, None)? };

        Ok(Self {
            layout
        })
    }
}

impl DescriptorSetLayout {
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

pub struct DescriptorPool {
    pub layout: DescriptorSetLayout,
    pub pool: vk::DescriptorPool,
    pub sets: Vec<vk::DescriptorSet>,
}

impl DescriptorPool {
    pub fn new(device: &ash::Device, uniform_buffer: &[Buffer]) -> VkResult<Self> {
        let layout = DescriptorSetLayout::new(device)?;

        let pool_size = vk::DescriptorPoolSize::builder()
            .descriptor_count(config::MAX_FRAMES_IN_FLIGHT)
            .build();

        let pool_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(&[pool_size])
            .max_sets(config::MAX_FRAMES_IN_FLIGHT)
            .build();

        let pool = unsafe{ device.create_descriptor_pool(&pool_info, None)? };

        let sets = create_descriptor_sets(device, pool, layout.layout, uniform_buffer)?;

        Ok(Self {
            layout,
            sets,
            pool,
        })
    }
}

impl DescriptorPool {
    pub fn drop_manual(&self, device: &ash::Device) {
        println!("> dropping DescriptorPool...");

        self.layout.drop_manual(device);

        unsafe {
            // automatically cleans up all associated sets
            device.destroy_descriptor_pool(self.pool, None);
        }
    }
}

fn create_descriptor_sets(device: &ash::Device, pool: vk::DescriptorPool, layout: vk::DescriptorSetLayout, uniform_buffer: &[Buffer]) -> VkResult<Vec<vk::DescriptorSet>> {
    let layouts = [layout; config::MAX_FRAMES_IN_FLIGHT as usize];

    let alloc_info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(pool)
        .set_layouts(&layouts)
        .build();

    let sets = unsafe { device.allocate_descriptor_sets(&alloc_info)? };

    for (idx, s) in sets.iter().enumerate() {
        let buffer_info = vk::DescriptorBufferInfo::builder()
            .buffer(uniform_buffer[idx].buffer)
            .offset(0)
            .range(std::mem::size_of::<UniformBufferObject>() as vk::DeviceSize)
            .build();

        let mut write_descriptor = vk::WriteDescriptorSet::builder()
            .dst_set(*s)
            .dst_binding(0) // corresponds to shader binding
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(&[buffer_info])
            .build();

        // TODO: check why this can't be set through builder
        write_descriptor.descriptor_count = 1;

        unsafe {
            device.update_descriptor_sets(&[write_descriptor], &[]);
        }
    }



    Ok(sets)
}
