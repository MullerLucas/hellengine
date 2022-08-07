use hell_common::window::{HellWindow, HellWindowExtent};
use hell_renderer::vulkan;



pub struct HellApp {
    renderer_2d: vulkan::Renderer2D,
}


// create
impl HellApp {
    pub fn new(window: &dyn HellWindow) -> Self {
        let surface_info = window.create_surface_info();
        let window_extent = window.get_window_extent();
        let core = vulkan::Core::new(&surface_info, &window_extent).unwrap();
        let renderer_2d = vulkan::Renderer2D::new(core);

        Self {
            renderer_2d,
        }
    }
}

// drop
impl Drop for HellApp {
    fn drop(&mut self) {
        println!("> dropping HellApp...");

        // TODO: implement drop
        // self.pipeline.drop_manual(&self.core.device.device);
        // drop (renderer_2d);
        // drop (core);
    }
}

// utils
impl HellApp {
    pub fn on_window_changed(&mut self, window_extent: &HellWindowExtent) {
        self.wait_idle();
        self.renderer_2d.wait_idle();
        self.renderer_2d.on_window_changed(window_extent);
    }

    pub fn wait_idle(&self) {
        self.renderer_2d.wait_idle();
    }

    pub fn draw_frame(&mut self, delta_time: f32) -> bool {
        self.renderer_2d.draw_frame(delta_time)
    }
}
