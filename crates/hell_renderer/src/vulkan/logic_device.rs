use std::{ptr, ffi};

use ash::vk;

use super::config;
use super::phys_device::VulkanPhysDevice;
use super::queues::VulkanQueues;


pub struct VulkanLogicDevice {
    pub device: ash::Device,
    pub queues: VulkanQueues,
}


impl VulkanLogicDevice {
    pub fn new(
        instance: &ash::Instance,
        phys_device: &VulkanPhysDevice
    ) -> Self {

        let queue_priorities = [1.0_f32];

        let queue_create_infos: Vec<_> = phys_device.queue_support
            .indices()
            .into_iter()
            .map(|idx| vk::DeviceQueueCreateInfo {
                s_type: vk::StructureType::DEVICE_QUEUE_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::DeviceQueueCreateFlags::empty(),
                queue_family_index: idx,
                queue_count: 1,
                p_queue_priorities: queue_priorities.as_ptr(),
            })
            .collect();

        let phys_device_features = vk::PhysicalDeviceFeatures::builder()
            .sampler_anisotropy(true)
            .sample_rate_shading(config::ENABLE_SAMPLE_SHADING)   // Sample-Shading
            .build();

        let extension_names: Vec<_> = config::DEVICE_EXTENSION_NAMES
            .iter()
            .map(|n| ffi::CString::new(*n).unwrap())
            .collect();

        let extension_names_input: Vec<_> = extension_names.iter().map(|n| n.as_ptr()).collect();

        let mut logic_device_create_info = vk::DeviceCreateInfo {
            s_type: vk::StructureType::DEVICE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::DeviceCreateFlags::empty(),
            queue_create_info_count: queue_create_infos.len() as u32,
            p_queue_create_infos: queue_create_infos.as_ptr(),
            // layers: ignored by modern implementations - add anyway, for backwards compatibility
            enabled_layer_count: 0,
            pp_enabled_layer_names: ptr::null(),
            // extensions: device-specific
            enabled_extension_count: config::DEVICE_EXTENSION_NAMES.len() as u32,
            pp_enabled_extension_names: extension_names_input.as_ptr(),
            p_enabled_features: &phys_device_features,
        };

        let validation_layer_names: Vec<_> = config::VALIDATION_LAYER_NAMES
            .iter()
            .map(|l| ffi::CString::new(*l).unwrap())
            .collect();

        let validation_layer_names_input: Vec<_> =
            validation_layer_names.iter().map(|l| l.as_ptr()).collect();

        if config::ENABLE_VALIDATION_LAYERS {
            logic_device_create_info.enabled_layer_count =
                validation_layer_names_input.len() as u32;
            logic_device_create_info.pp_enabled_layer_names = validation_layer_names_input.as_ptr();
        }

        let device = unsafe {
            instance
                .create_device(phys_device.device, &logic_device_create_info, None)
                .expect("failed to create logical device")
        };

        let queues = VulkanQueues::from_support(&device, &phys_device.queue_support);

        Self {
            device,
            queues
        }
    }
}
