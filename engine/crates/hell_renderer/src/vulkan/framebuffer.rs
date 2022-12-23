use std::array;

use ash::vk;
use hell_error::HellResult;
use hell_core::config;

use crate::shared::collections::PerFrame;

use super::VulkanCtxRef;
use super::image::DepthImage;
use super::render_pass::VulkanRenderPass;
use super::swapchain::VulkanSwapchain;


pub struct VulkanFramebuffer {
    ctx: VulkanCtxRef,
    handles: PerFrame<vk::Framebuffer>,
}

impl Drop for VulkanFramebuffer {
    fn drop(&mut self) {
        println!("> dropping Framebuffer...");

        unsafe {
            let device = &self.ctx.device.device;
            self.handles.get_all().iter().for_each(|b| {
                device.destroy_framebuffer(*b, None);
            });
        }
    }
}

impl VulkanFramebuffer {

    pub fn new_world_buffer(ctx: &VulkanCtxRef, swapchain: &VulkanSwapchain, render_pass: &VulkanRenderPass, depth_buffer: &DepthImage,) -> HellResult<Self> {
        let buffers = array::from_fn(|idx| {
            // only a single subpass is running at the same time, so we can reuse the same depth-buffer for all frames in flight
            let sv = &swapchain.views[idx];
            let attachments = [*sv, depth_buffer.img.view];

            let buffer_info = vk::FramebufferCreateInfo::builder()
                .render_pass(render_pass.handle)
                .attachments(&attachments) // sets count
                .width(swapchain.extent.width)
                .height(swapchain.extent.height)
                .layers(config::FRAME_BUFFER_LAYER_COUNT)
                .build();

            // TODO: no unwrap
            unsafe { ctx.device.device.create_framebuffer(&buffer_info, None).unwrap() }
        });


        Ok(Self {
            ctx: ctx.clone(),
            handles: PerFrame::new(buffers)
        })
    }

    pub fn new_ui_buffer(ctx: &VulkanCtxRef, swapchain: &VulkanSwapchain, render_pass: &VulkanRenderPass) -> HellResult<Self> {
        let buffers = array::from_fn(|idx| {
            // only a single subpass is running at the same time, so we can reuse the same depth-buffer for all frames in flight
            let sv = &swapchain.views[idx];
            let attachments = [*sv];

            let buffer_info = vk::FramebufferCreateInfo::builder()
                .render_pass(render_pass.handle)
                .attachments(&attachments) // sets count
                .width(swapchain.extent.width)
                .height(swapchain.extent.height)
                .layers(config::FRAME_BUFFER_LAYER_COUNT)
                .build();

            // TODO: no unwrap
            unsafe { ctx.device.device.create_framebuffer(&buffer_info, None).unwrap() }
        });


        Ok(Self {
            ctx: ctx.clone(),
            handles: PerFrame::new(buffers)
        })
    }
}

impl VulkanFramebuffer {
    pub fn buffer_at(&self, img_idx: usize) -> vk::Framebuffer {
        self.handles.get(img_idx).clone()
    }
}
