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
        // color_img_view: vk::ImageView,
        render_pass: &VulkanRenderPass,
    ) -> Self {

        let buffers = swapchain.img_views
            .iter()
            .map(|sv| {

                // let attachments = [color_img_view, *sv];
                let attachments = [*sv];

                let buffer_info = vk::FramebufferCreateInfo::builder()
                    .render_pass(render_pass.render_pass)
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
    pub fn drop_manual(&self, device: &ash::Device) {
        println!("> dropping Framebuffer...");

        unsafe {
            self.buffers.iter().for_each(|b| {
                device.destroy_framebuffer(*b, None);
            });
        }
    }
}

impl VulkanFramebuffer {
    pub fn buffer_at(&self, img_idx: usize) -> vk::Framebuffer {
        self.buffers[img_idx]
    }
}
