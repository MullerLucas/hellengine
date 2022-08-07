use ash::vk;

use super::config;
use super::render_pass::RenderPass;
use super::swapchain::Swapchain;



pub struct Framebuffer {
    buffers: Vec<vk::Framebuffer>,
}

impl Framebuffer {

    pub fn new(
        device: &ash::Device,
        swapchain: &Swapchain,
        // color_img_view: vk::ImageView,
        render_pass: &RenderPass,
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

impl Framebuffer {
    pub fn drop_manual(&self, device: &ash::Device) {
        println!("> dropping Framebuffer...");

        unsafe {
            self.buffers.iter().for_each(|b| {
                device.destroy_framebuffer(*b, None);
            });
        }
    }
}

impl Framebuffer {
    pub fn buffer_at(&self, img_idx: usize) -> vk::Framebuffer {
        self.buffers[img_idx]
    }
}
