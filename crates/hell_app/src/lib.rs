use hell_common::window::{HellWindow, HellWindowExtent};
use hell_renderer::vulkan;
use hell_renderer::vulkan::VulkanCamera;
use crate::scene::Scene;

mod scene;



pub struct HellApp {
    scene: Scene,
    camera: VulkanCamera,
    renderer_2d: vulkan::VulkanRenderer2D,
}


// create
impl HellApp {
    pub fn new(window: &dyn HellWindow) -> Self {
        let surface_info = window.create_surface_info();
        let window_extent = window.get_window_extent();
        let core = vulkan::VulkanCore::new(&surface_info, &window_extent).unwrap();
        let camera = VulkanCamera::new(&core);
        let renderer_2d = vulkan::VulkanRenderer2D::new(core);

        Self {
            scene: Scene::default(),
            camera,
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
        // TODO: remove
        // std::thread::sleep(std::time::Duration::from_millis(250));
        // let delta_time = 0.1;

        self.scene.update(delta_time);

        let frame_data = &self.renderer_2d.frame_data;
        let camera_buffer = &frame_data.camera_ubos.get(self.renderer_2d.curr_frame_idx).unwrap();
        self.camera.update_uniform_buffer(&self.renderer_2d.core, delta_time, camera_buffer);

        self.renderer_2d.draw_frame(delta_time, self.scene.get_render_data())
    }
}
