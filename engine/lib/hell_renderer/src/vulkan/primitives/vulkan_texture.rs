use hell_core::error::HellResult;

use crate::vulkan::VulkanContextRef;

use super::{VulkanImage, VulkanSampler, VulkanCommands};

pub struct VulkanTexture {
    pub img: VulkanImage,
    pub sampler: VulkanSampler,
}

impl VulkanTexture {
    pub fn new(ctx: &VulkanContextRef, cmds: &VulkanCommands, data: &[u8], width: usize, height: usize) -> HellResult<Self> {
        let img = VulkanImage::new_tex_img(ctx, cmds, data, width, height)?;
        let sampler = VulkanSampler::new(ctx)?;

        Ok(Self { img, sampler })
    }

    // TODO: improve
    pub fn new_default(ctx: &VulkanContextRef, cmds: &VulkanCommands) -> HellResult<Self> {
        let img = VulkanImage::new_tex_img_default(ctx, cmds)?;
        let sampler = VulkanSampler::new(ctx)?;

        Ok(Self {
            img, sampler
        })
    }
}

