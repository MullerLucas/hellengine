use ash::vk;

use hell_common::prelude::*;
use crate::vulkan::VulkanCore;

use super::RawImage;

pub struct DepthImage {
    pub img: RawImage,
}

impl DepthImage {
    pub fn new(core: &VulkanCore) -> HellResult<Self> {
        let depth_format = core.phys_device.depth_format;
        let extent = core.swapchain.extent;

        let img = RawImage::new(
            core,
            extent.width,
            extent.height,
            vk::SampleCountFlags::TYPE_1,
            depth_format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            vk::ImageAspectFlags::DEPTH
        );

        // Not required: Layout will be transitioned in the renderpass
        img.transition_image_layout(
            &core.device.device,
            &core.graphics_cmd_pool,
            &core.device.queues.graphics,
            depth_format,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        )?;

        Ok(Self { img })
    }
}

impl DepthImage {
    pub fn drop_manual(&self, device: &ash::Device) {
        println!("> dropping DepthImage...");

        self.img.drop_manual(device);
    }
}
