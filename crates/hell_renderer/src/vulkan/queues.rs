use std::collections::HashSet;
use ash::vk;

use super::surface::Surface;



// ----------------------------------------------------------------------------
// queue-famiyy
// ----------------------------------------------------------------------------

#[derive(Debug)]
pub struct QueueFamily {
    pub idx: u32,
    pub properties : vk::QueueFamilyProperties,
}

impl QueueFamily {
    pub fn new(idx: u32, properties: vk::QueueFamilyProperties) -> Self {
        Self { idx, properties }
    }
}



// ----------------------------------------------------------------------------
// queue
// ----------------------------------------------------------------------------

pub struct Queue {
    pub family_idx: u32,
    pub queue_idx: u32,
    pub vk_queue: vk::Queue,
}

impl Queue {
    pub fn new(device: &ash::Device, family_idx: u32, queue_idx: u32) -> Self {
        let vk_queue = unsafe { device.get_device_queue(family_idx, queue_idx) };

        Self {
            family_idx,
            queue_idx,
            vk_queue
        }
    }
}



// ----------------------------------------------------------------------------
// queues
// ----------------------------------------------------------------------------

pub struct Queues {
    pub graphics: Queue,
    pub present: Queue,
    pub transfer: Queue,
}

impl Queues {
    pub fn from_support(device: &ash::Device, support: &QueueSupport) -> Self {
        let graphics_family = support.graphics_family.as_ref().unwrap();
        let present_family = support.present_family.as_ref().unwrap();
        let transfer_family = support.transfer_family.as_ref().unwrap();

        let graphics_queue = Queue::new(device, graphics_family.idx, 0);
        let present_queue = Queue::new(device, present_family.idx, 0);
        let transfer_queue = Queue::new(device, transfer_family.idx, 0);

        Self {
            graphics: graphics_queue,
            present: present_queue,
            transfer: transfer_queue,
        }
    }
}




// ----------------------------------------------------------------------------
// queue-support
// ----------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct QueueSupport {
    pub graphics_family: Option<QueueFamily>,
    pub present_family: Option<QueueFamily>,
    pub transfer_family: Option<QueueFamily>,
}

impl QueueSupport {
    pub fn new(instance: &ash::Instance, phys_device: vk::PhysicalDevice, surface_data: &Surface) -> Self {
        let properties = unsafe { instance.get_physical_device_queue_family_properties(phys_device) };


        let mut result = QueueSupport::default();
        for (idx , prop) in properties.into_iter().enumerate() {
            let idx = idx as u32;

            if prop.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                result.graphics_family = Some(QueueFamily::new(idx, prop));
            } else if prop.queue_flags.contains(vk::QueueFlags::TRANSFER) {
                result.transfer_family = Some(QueueFamily::new(idx, prop));
            }

            if result.present_family.is_none() {
                let present_is_supported = unsafe {
                    surface_data.surface_loader
                        .get_physical_device_surface_support(phys_device, idx, surface_data.surface)
                        .unwrap()
                };

                if present_is_supported {
                    result.present_family = Some(QueueFamily::new(idx, prop));
                }
            }

            if result.is_complete() { break; }
        }


        result
    }

}

impl QueueSupport {
    // TODO:
    pub fn single_queue(&self) -> bool {
        self.graphics_family.as_ref().unwrap().idx == self.present_family.as_ref().unwrap().idx
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
