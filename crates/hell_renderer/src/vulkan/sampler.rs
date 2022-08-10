use ash::prelude::VkResult;
pub use ash::vk;

use super::VulkanCore;

pub struct VulkanSampler {
    pub sampler: vk::Sampler,
}



impl VulkanSampler {
    pub fn new(core: &VulkanCore) -> VkResult<Self> {

        // enabled ansiotropy if the physical device supports it
        let (ansiotropy_enabled, max_ansiotropy) = if core.phys_device.features.sampler_anisotropy == vk::TRUE {
            let max_ansiotropy = core.phys_device.device_props.limits.max_sampler_anisotropy;
            (true, max_ansiotropy)
        } else {
            (false, 1.0)
        };

        let sampler_info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR)
            .address_mode_u(vk::SamplerAddressMode::REPEAT)
            .address_mode_v(vk::SamplerAddressMode::REPEAT)
            .address_mode_w(vk::SamplerAddressMode::REPEAT)
            .anisotropy_enable(ansiotropy_enabled)
            .max_anisotropy(max_ansiotropy)
            .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
            .unnormalized_coordinates(false)
            .compare_enable(false)
            .compare_op(vk::CompareOp::ALWAYS)
            .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
            .mip_lod_bias(0.0)
            .min_lod(0.0)
            .max_lod(0.0)
            .build();

        let sampler = unsafe { core.device.device.create_sampler(&sampler_info, None)? };

        Ok(Self {
            sampler
        })
    }
}

impl VulkanSampler {
    pub fn drop_manual(&self, device: &ash::Device) {
        println!("> dropping VulkanTextureSampler...");

        unsafe {
            device.destroy_sampler(self.sampler, None);
        }
    }
}
