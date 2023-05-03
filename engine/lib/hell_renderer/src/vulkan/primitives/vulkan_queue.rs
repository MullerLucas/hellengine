use std::collections::HashSet;
use ash::vk;
use hell_core::error::{HellResult, OptToHellErr};
use super::VulkanSurface;





// ----------------------------------------------------------------------------
// queue-famiyy
// ----------------------------------------------------------------------------

#[derive(Debug)]
pub struct VulkanQueueFamily {
    pub idx: u32,
    pub properties : vk::QueueFamilyProperties,
}

impl VulkanQueueFamily {
    pub fn new(idx: u32, properties: vk::QueueFamilyProperties) -> Self {
        Self { idx, properties }
    }
}



// ----------------------------------------------------------------------------
// queue
// ----------------------------------------------------------------------------

pub struct VulkanQueue {
    pub family_idx: u32,
    pub queue_idx: u32,
    pub queue: vk::Queue,
}

impl VulkanQueue {
    pub fn new(device: &ash::Device, family_idx: u32, queue_idx: u32) -> Self {
        let vk_queue = unsafe { device.get_device_queue(family_idx, queue_idx) };

        Self {
            family_idx,
            queue_idx,
            queue: vk_queue
        }
    }
}



// ----------------------------------------------------------------------------
// queues
// ----------------------------------------------------------------------------

pub struct VulkanQueues {
    pub graphics: VulkanQueue,
    pub present: VulkanQueue,
    pub transfer: VulkanQueue,
}

impl VulkanQueues {
    pub fn from_support(device: &ash::Device, support: &VulkanQueueSupport) -> HellResult<Self> {
        let graphics_family = support.graphics_family.as_ref().to_render_hell_err()?;
        let present_family = support.present_family.as_ref().to_render_hell_err()?;
        let transfer_family = support.transfer_family.as_ref().to_render_hell_err()?;

        let graphics_queue = VulkanQueue::new(device, graphics_family.idx, 0);
        let present_queue = VulkanQueue::new(device, present_family.idx, 0);
        let transfer_queue = VulkanQueue::new(device, transfer_family.idx, 0);

        Ok(Self {
            graphics: graphics_queue,
            present: present_queue,
            transfer: transfer_queue,
        })
    }
}




// ----------------------------------------------------------------------------
// queue-support
// ----------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct VulkanQueueSupport {
    pub graphics_family: Option<VulkanQueueFamily>,
    pub present_family: Option<VulkanQueueFamily>,
    pub transfer_family: Option<VulkanQueueFamily>,
}

impl VulkanQueueSupport {
    pub fn new(instance: &ash::Instance, phys_device: vk::PhysicalDevice, surface_data: &VulkanSurface) -> HellResult<Self> {
        let properties = unsafe { instance.get_physical_device_queue_family_properties(phys_device) };


        let mut result = VulkanQueueSupport::default();
        for (idx , prop) in properties.into_iter().enumerate() {
            let idx = idx as u32;

            if prop.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                result.graphics_family = Some(VulkanQueueFamily::new(idx, prop));
            } else if prop.queue_flags.contains(vk::QueueFlags::TRANSFER) {
                result.transfer_family = Some(VulkanQueueFamily::new(idx, prop));
            }

            if result.present_family.is_none() {
                let present_is_supported = unsafe {
                    surface_data.surface_loader
                        .get_physical_device_surface_support(phys_device, idx, surface_data.surface)?
                };

                if present_is_supported {
                    result.present_family = Some(VulkanQueueFamily::new(idx, prop));
                }
            }

            if result.is_complete() { break; }
        }


        Ok(result)
    }

}

impl VulkanQueueSupport {
    pub fn single_queue(&self) -> HellResult<bool> {
        Ok(
            self.graphics_family.as_ref().to_render_hell_err()?.idx == self.present_family.as_ref().to_render_hell_err()?.idx
        )
    }

    pub fn indices(&self) -> HashSet<u32> {
        [self.graphics_family.as_ref(), self.present_family.as_ref(), self.transfer_family.as_ref()].into_iter()
            .flatten()
            .map(|f| f.idx)
            .collect()
    }

    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some() &&
            self.present_family.is_some() &&
            self.transfer_family.is_some()
    }

    pub fn print_queue_families(instance: &ash::Instance, device: vk::PhysicalDevice) {
        let props = unsafe { instance.get_physical_device_queue_family_properties(device) };

        for (idx, prop) in props.iter().enumerate() {
            println!("\t> queue: {}", idx);

            if prop.queue_flags.contains(vk::QueueFlags::GRAPHICS) { println!("\t\t> GRAPHICS-QUEUE"); }
            if prop.queue_flags.contains(vk::QueueFlags::COMPUTE) { println!("\t\t> COMPUTE-QUEUE"); }
            if prop.queue_flags.contains(vk::QueueFlags::TRANSFER) { println!("\t\t> TRANSFER-QUEUE"); }
            if prop.queue_flags.contains(vk::QueueFlags::SPARSE_BINDING) { println!("\t\t> SPARSE-BINDING-QUEUE"); }
        }
    }
}
