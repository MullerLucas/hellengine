use ash::prelude::VkResult;
pub use ash::vk;

use crate::vulkan::VulkanContextRef;




pub struct VulkanSampler {
    ctx: VulkanContextRef,
    pub handle: vk::Sampler,
}

impl Drop for VulkanSampler {
    fn drop(&mut self) {
        println!("> dropping VulkanTextureSampler...");

        unsafe {
            let device = &self.ctx.device.handle;
            device.destroy_sampler(self.handle, None);
        }
    }
}


impl VulkanSampler {
    pub fn new(ctx: &VulkanContextRef) -> VkResult<Self> {

        // enabled ansiotropy if the physical device supports it
        let (ansiotropy_enabled, max_ansiotropy) = if ctx.phys_device.features.sampler_anisotropy == vk::TRUE {
            let max_ansiotropy = ctx.phys_device.device_props.limits.max_sampler_anisotropy;
            (true, max_ansiotropy)
        } else {
            (false, 1.0)
        };

        let sampler_info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::NEAREST)
            .min_filter(vk::Filter::NEAREST)
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

        let sampler = unsafe { ctx.device.handle.create_sampler(&sampler_info, None)? };

        Ok(Self {
            ctx: ctx.clone(),
            handle: sampler
        })
    }
}
