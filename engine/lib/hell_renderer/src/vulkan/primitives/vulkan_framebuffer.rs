use ash::vk;
use hell_core::error::HellResult;
use crate::{vulkan::VulkanContextRef, config};

use super::{VulkanSwapchain, VulkanImage, VulkanRenderPass};


pub struct VulkanFramebuffer {
    ctx: VulkanContextRef,
    handles: Vec<vk::Framebuffer>, // one per Swapchain-Image
}

impl Drop for VulkanFramebuffer {
    fn drop(&mut self) {
        println!("> dropping Framebuffer...");

        unsafe {
            let device = &self.ctx.device.handle;
            self.handles.iter().for_each(|b| {
                device.destroy_framebuffer(*b, None);
            });
        }
    }
}

impl VulkanFramebuffer {

    pub fn new_world_buffer(ctx: &VulkanContextRef, swapchain: &VulkanSwapchain, render_pass: &VulkanRenderPass, depth_buffer: &VulkanImage) -> HellResult<Self> {
        // let buffers = array::from_fn(|idx| {
        let handles = swapchain.views.iter()
            .map(|sv| {
                // only a single subpass is running at the same time, so we can reuse the same depth-buffer for all frames in flight
                // let sv = &swapchain.views[idx];
                let attachments = [*sv, depth_buffer.view];

                let buffer_info = vk::FramebufferCreateInfo::builder()
                    .render_pass(render_pass.handle)
                    .attachments(&attachments) // sets count
                    .width(swapchain.extent.width)
                    .height(swapchain.extent.height)
                    .layers(config::FRAME_BUFFER_LAYER_COUNT)
                    .build();

                // TODO: no unwrap
                unsafe { ctx.device.handle.create_framebuffer(&buffer_info, None).unwrap() }
            })
            .collect();



        Ok(Self {
            ctx: ctx.clone(),
            handles,
        })
    }

    pub fn new_ui_buffer(ctx: &VulkanContextRef, swapchain: &VulkanSwapchain, render_pass: &VulkanRenderPass) -> HellResult<Self> {
        // let buffers = array::from_fn(|idx| {
        let handles = swapchain.views.iter()
            .map(|sv| {
                // only a single subpass is running at the same time, so we can reuse the same depth-buffer for all frames in flight
                // let sv = &swapchain.views[idx];
                let attachments = [*sv];

                let buffer_info = vk::FramebufferCreateInfo::builder()
                    .render_pass(render_pass.handle)
                    .attachments(&attachments) // sets count
                    .width(swapchain.extent.width)
                    .height(swapchain.extent.height)
                    .layers(config::FRAME_BUFFER_LAYER_COUNT)
                    .build();

                // TODO: no unwrap
                unsafe { ctx.device.handle.create_framebuffer(&buffer_info, None).unwrap() }
            })
            .collect();


        Ok(Self {
            ctx: ctx.clone(),
            handles,
        })
    }
}

impl VulkanFramebuffer {
    pub fn buffer_at(&self, img_idx: usize) -> vk::Framebuffer {
        self.handles[img_idx]
    }
}
