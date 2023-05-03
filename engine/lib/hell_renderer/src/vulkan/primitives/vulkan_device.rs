
use std::{ptr, ffi};

use ash::vk;
use hell_core::error::{HellResult, ErrToHellErr};
use hell_utils::conversion;

use crate::config;

use super::{VulkanPhysDevice, VulkanQueues};




pub struct VulkanLogicDevice {
    pub handle: ash::Device,
    pub queues: VulkanQueues,
}

impl VulkanLogicDevice {
    pub fn new(instance: &ash::Instance, phys_device: &VulkanPhysDevice) -> HellResult<Self> {

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

        let raw_extension_names = conversion::c_char_from_str_slice(config::DEVICE_EXTENSION_NAMES)?;

        let mut logic_device_create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(&queue_create_infos)
            // extensions: device-specific
            .enabled_extension_names(&raw_extension_names.1)
            .enabled_features(&phys_device_features)
            .push_next(&mut phys_device_feature_11)
            .build();

        let validation_layer_names: HellResult<Vec<_>> = config::VALIDATION_LAYER_NAMES
            .iter()
            .map(|l| ffi::CString::new(*l).to_render_hell_err())
            .collect();
        let validation_layer_names = validation_layer_names?;

        let validation_layer_names_input: Vec<_> =
            validation_layer_names.iter().map(|l| l.as_ptr()).collect();

        if config::ENABLE_VALIDATION_LAYERS {
            logic_device_create_info.enabled_layer_count =
                validation_layer_names_input.len() as u32;
            logic_device_create_info.pp_enabled_layer_names = validation_layer_names_input.as_ptr();
        }

        let device = unsafe { instance.create_device(phys_device.phys_device, &logic_device_create_info, None)? };
        let queues = VulkanQueues::from_support(&device, &phys_device.queue_support)?;

        Ok(Self {
            handle: device,
            queues
        })
    }
}

impl Drop for VulkanLogicDevice {
    fn drop(&mut self) {
        println!("> dropping LogicDevice...");

        unsafe {
            // cleans up device queues
            self.handle.destroy_device(None);
        }
    }
}

impl VulkanLogicDevice {
    pub fn wait_idle(&self) -> HellResult<()> {
        unsafe { self.handle.device_wait_idle().to_render_hell_err()?; }

        Ok(())
    }
}
