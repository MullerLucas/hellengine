
use std::array;

use ash::prelude::VkResult;
use ash::vk;
use hell_core::error::HellResult;
use crate::render_types::PerFrame;
use crate::vulkan::VulkanContextRef;






// ----------------------------------------------------------------------------
// descriptor-pool
// ----------------------------------------------------------------------------

pub struct VulkanDescriptorSetGroup {
    ctx: VulkanContextRef,
    pub layout: vk::DescriptorSetLayout,
    pub handles: Vec<PerFrame<vk::DescriptorSet>>,
}

impl Drop for VulkanDescriptorSetGroup {
    fn drop(&mut self) {
        println!("> dropping VulkanDescriptorSetLayoutGroup...");

        unsafe {
            let device = &self.ctx.device.handle;
            device.destroy_descriptor_set_layout(self.layout, None);
        }
    }
}

impl VulkanDescriptorSetGroup {
    pub fn new(ctx: &VulkanContextRef, layout: vk::DescriptorSetLayout, capacity: usize) -> Self {
        Self {
            ctx: ctx.clone(),
            layout,
            handles: Vec::with_capacity(capacity)
        }
    }

    pub fn create_descriptor_set_layout(device: &ash::Device, bindings: &[vk::DescriptorSetLayoutBinding]) -> VkResult<vk::DescriptorSetLayout> {
        let layout_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(bindings)
            .build();

        Ok(unsafe {
            device.create_descriptor_set_layout(&layout_info, None)?
        })
    }

    pub fn allocate_sets_for_layout(ctx: &VulkanContextRef, layout: vk::DescriptorSetLayout, pool: vk::DescriptorPool) -> HellResult<PerFrame<vk::DescriptorSet>> {
        let layouts: PerFrame<vk::DescriptorSetLayout> = array::from_fn(|_| layout);

        // create sets
        // -----------
        let alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(pool)
            .set_layouts(&layouts)
            .build();

        let sets = unsafe { ctx.device.handle.allocate_descriptor_sets(&alloc_info)? };
        let sets: PerFrame<vk::DescriptorSet> = array::from_fn(|idx| sets[idx]);

        Ok(sets)
    }
}

impl VulkanDescriptorSetGroup {
    pub fn set_count(&self) -> usize {
        self.handles.len()
    }
}
