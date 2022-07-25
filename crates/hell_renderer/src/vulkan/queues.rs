use std::collections::HashSet;
use ash::vk;

use super::surface::VulkanSurface;


#[derive(Debug, Default)]
pub struct VulkanQueueSupport {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
    pub transfer_family: Option<u32>,
}

impl VulkanQueueSupport {
    pub fn new(instance: &ash::Instance, phys_device: vk::PhysicalDevice, surface_data: &VulkanSurface) -> Self {
        let props = unsafe { instance.get_physical_device_queue_family_properties(phys_device) };


        let mut result = VulkanQueueSupport::default();
        for (idx , prop) in props.iter().enumerate() {
            let idx = idx as u32;

            if prop.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                result.graphics_family = Some(idx);
            } else if prop.queue_flags.contains(vk::QueueFlags::TRANSFER) {
                result.transfer_family = Some(idx);
            }

            if result.present_family.is_none() {
                let present_is_supported = unsafe {
                    surface_data.surface_loader
                        .get_physical_device_surface_support(phys_device, idx, surface_data.surface)
                        .unwrap()
                };

                if present_is_supported {
                    result.present_family = Some(idx);
                }
            }

            if result.is_complete() { break; }
        }


        result
    }

}

impl VulkanQueueSupport {
    pub fn single_queue(&self) -> bool {
        self.graphics_family.unwrap() == self.present_family.unwrap()
    }

    pub fn indices(&self) -> HashSet<u32> {
        [self.graphics_family, self.present_family, self.transfer_family].into_iter()
            .flatten()
            .collect()
    }

    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some() &&
            self.present_family.is_some() &&
            self.transfer_family.is_some()
    }
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



pub struct VulkanQueues {
    pub graphics_idx: u32,
    pub graphics_queue: vk::Queue,
    pub _present_idx: u32,
    pub present_queue: vk::Queue,
    pub transfer_idx: u32,
    pub transfer_queue: vk::Queue,
}

impl VulkanQueues {
    pub fn from_support(device: &ash::Device, support: &VulkanQueueSupport) -> Self {
        let graphics_idx = support.graphics_family.unwrap();
        let present_idx = support.present_family.unwrap();
        let transfer_idx = support.transfer_family.unwrap();

        unsafe {
            Self {
                graphics_idx,
                graphics_queue: device.get_device_queue(graphics_idx, 0),
                _present_idx: present_idx,
                present_queue: device.get_device_queue(present_idx, 0),
                transfer_idx,
                transfer_queue: device.get_device_queue(transfer_idx, 0),
            }
        }
    }
}
