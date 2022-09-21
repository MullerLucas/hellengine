use hell_common::transform::Transform;
use hell_common::window::{HellWindow, HellWindowExtent};
use hell_renderer::vulkan;



pub struct HellApp {
    renderer_2d: vulkan::VulkanRenderer2D,
    trans_1: Transform,
    trans_2: Transform,
}


// create
impl HellApp {
    pub fn new(window: &dyn HellWindow) -> Self {
        let surface_info = window.create_surface_info();
        let window_extent = window.get_window_extent();
        let core = vulkan::VulkanCore::new(&surface_info, &window_extent).unwrap();
        let renderer_2d = vulkan::VulkanRenderer2D::new(core);

        Self {
            renderer_2d,
            trans_1: Transform::default(),
            trans_2: Transform::default(),
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
        // TODO: remove
        std::thread::sleep(std::time::Duration::from_millis(1000));
        let delta_time = 0.1;

        self.trans_1.scale_uniform(1f32 + delta_time / 5f32);
        self.trans_1.rotate_around_z((delta_time * 30f32).to_radians());
        self.trans_1.translate_x(delta_time);

        self.trans_2.scale_uniform(1f32 - delta_time / 5f32);
        self.trans_2.rotate_around_z((delta_time * -30f32).to_radians());
        self.trans_2.translate_x(-delta_time);

        self.renderer_2d.uniform_data.update_uniform_buffer(&self.renderer_2d.core, self.renderer_2d.curr_frame_idx as usize, delta_time);

        self.renderer_2d.draw_frame(delta_time, &[&self.trans_1, &self.trans_2])
    }
}
