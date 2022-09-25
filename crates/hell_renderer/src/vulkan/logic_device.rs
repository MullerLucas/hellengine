use std::{ptr, ffi};

use ash::vk;
use hell_utils::conversion;

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
        let mut phys_device_feature_11 = vk::PhysicalDeviceVulkan11Features::builder()
            .shader_draw_parameters(true)
            .build();

        let raw_extension_names = conversion::c_char_from_str_slice(config::DEVICE_EXTENSION_NAMES);

        let mut logic_device_create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_create_infos)
            // extensions: device-specific
            .enabled_extension_names(&raw_extension_names.1)
            .enabled_features(&phys_device_features)
            .push_next(&mut phys_device_feature_11)
            .build();

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
                .create_device(phys_device.phys_device, &logic_device_create_info, None)
                .expect("failed to create logical device")
        };

        let queues = VulkanQueues::from_support(&device, &phys_device.queue_support);

        Self {
            device,
            queues
        }
    }
}

impl Drop for VulkanLogicDevice {
    fn drop(&mut self) {
        println!("> dropping LogicDevice...");

        unsafe {
            // cleans up device queues
            self.device.destroy_device(None);
        }
    }
}

impl VulkanLogicDevice {
    // TODO: error handling
    pub fn wait_idle(&self) {
        unsafe {
            self.device.device_wait_idle().unwrap();
        }
    }
}
