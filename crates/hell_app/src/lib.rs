use hell_common::window::{HellWindow, HellWindowExtent};
use hell_renderer::vulkan::pipeline::VulkanGraphicsPipeline;
use hell_renderer::vulkan::vulkan_core::VulkanCore;

pub struct HellApp {
    core: VulkanCore,
    pipeline: VulkanGraphicsPipeline,
}

impl HellApp {
    pub fn new(window: &dyn HellWindow) -> Self {
        let surface_info = window.create_surface_info();
        let window_extent = window.get_window_extent();
        let core = VulkanCore::new(&surface_info, &window_extent).unwrap();
        let pipeline = VulkanGraphicsPipeline::new(&core);

        Self {
            core,
            pipeline
        }
    }

    pub fn on_window_changed(&mut self, window_extent: &HellWindowExtent) {
        self.wait_idle();
        self.core.recreate_swapchain(window_extent);
        self.pipeline.recreate_framebuffer(&self.core);
    }

    pub fn wait_idle(&self) {
        self.core.wait_device_idle();
    }

    pub fn draw_frame(&mut self, delta_time: f32) -> bool {
        self.core.draw_frame(&self.pipeline, delta_time)
    }
}

impl Drop for HellApp {
    fn drop(&mut self) {
        println!("> dropping HellApp...");

        // TODO: implement drop
        self.pipeline.drop_manual(&self.core.device.device);
        // self.core.drop_manual();
    }
}
