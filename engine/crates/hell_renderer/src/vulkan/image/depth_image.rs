use ash::vk;
use hell_error::HellResult;

use crate::vulkan::{VulkanCtxRef, VulkanSwapchain, command_buffer::VulkanCommands};

use super::RawImage;

pub struct DepthImage {
    pub img: RawImage,
}

impl DepthImage {
    pub fn new(ctx: &VulkanCtxRef, swapchain: &VulkanSwapchain, cmds: &VulkanCommands) -> HellResult<Self> {
        let depth_format = ctx.phys_device.depth_format;
        let extent = swapchain.extent;

        let img = RawImage::new(
            ctx,
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
            &ctx.device.device,
            &cmds.graphics_pool,
            &ctx.device.queues.graphics,
            depth_format,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        )?;

        Ok(Self { img })
    }
}
