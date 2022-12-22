use ash::vk;
use hell_error::{HellResult, ErrToHellErr};
use hell_core::config;

use super::VulkanCtxRef;
use super::image::DepthImage;
use super::render_pass::VulkanRenderPass;
use super::swapchain::VulkanSwapchain;



pub struct VulkanFramebuffer {
    ctx: VulkanCtxRef,
    buffers: Vec<vk::Framebuffer>,
}

impl Drop for VulkanFramebuffer {
    fn drop(&mut self) {
        println!("> dropping Framebuffer...");

        unsafe {
            let device = &self.ctx.device.device;
            self.buffers.iter().for_each(|b| {
                device.destroy_framebuffer(*b, None);
            });
        }
    }
}

impl VulkanFramebuffer {

    pub fn new(ctx: &VulkanCtxRef, swapchain: &VulkanSwapchain, render_pass: &VulkanRenderPass, depth_buffer: &DepthImage,) -> HellResult<Self> {

        let buffers: Result<Vec<_>, _> = swapchain.views
            .iter()
            .map(|sv| {
                // only a single subpass is running at the same time, so we can reuse the same depth-buffer for all frames in flight
                let attachments = [*sv, depth_buffer.img.view];

                let buffer_info = vk::FramebufferCreateInfo::builder()
                    .render_pass(render_pass.render_pass)
                    .attachments(&attachments) // sets count
                    .width(swapchain.extent.width)
                    .height(swapchain.extent.height)
                    .layers(config::FRAME_BUFFER_LAYER_COUNT)
                    .build();

                unsafe { ctx.device.device.create_framebuffer(&buffer_info, None).to_render_hell_err() }

            })
            .collect();


        Ok(Self {
            ctx: ctx.clone(),
            buffers: buffers?
        })
    }
}

impl VulkanFramebuffer {
    pub fn buffer_at(&self, img_idx: usize) -> vk::Framebuffer {
        self.buffers[img_idx]
    }
}
