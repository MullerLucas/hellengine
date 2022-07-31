use ash::vk;

use super::config;
use super::render_pass::VulkanRenderPass;
use super::swapchain::VulkanSwapchain;



pub struct VulkanFramebuffer {
    buffers: Vec<vk::Framebuffer>,
}

impl VulkanFramebuffer {

    pub fn new(
        device: &ash::Device,
        swapchain: &VulkanSwapchain,
        color_img_view: vk::ImageView,
        render_pass: &VulkanRenderPass,
    ) -> Self {

        let buffers = swapchain.img_views
            .iter()
            .map(|sv| {

                let attachments = [color_img_view, *sv];

                let buffer_info = vk::FramebufferCreateInfo::builder()
                    .render_pass(render_pass.pass)
                    .attachments(&attachments) // sets count
                    .width(swapchain.extent.width)
                    .height(swapchain.extent.height)
                    .layers(config::FRAME_BUFFER_LAYER_COUNT)
                    .build();

                // TODO: error handling
                unsafe { device.create_framebuffer(&buffer_info, None).unwrap() }

            })
            .collect();


        Self { buffers }
    }
}

impl VulkanFramebuffer {
    pub fn buffer_at(&self, idx: usize) -> vk::Framebuffer {
        self.buffers[idx]
    }
}
