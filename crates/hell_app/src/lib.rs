use hell_common::window::{HellWindow, HellWindowExtent};
use hell_renderer::vulkan::{self, ObjectData};
use hell_renderer::vulkan::CameraData;
use crate::scene::Scene;

mod scene;



pub struct HellApp {
    scene: Scene,
    camera_data: CameraData,
    renderer_2d: vulkan::VulkanRenderer2D,
}


// create
impl HellApp {
    pub fn new(window: &dyn HellWindow) -> Self {
        let surface_info = window.create_surface_info();
        let window_extent = window.get_window_extent();
        let core = vulkan::VulkanCore::new(&surface_info, &window_extent).unwrap();
        let camera = CameraData::new(&core);
        let renderer_2d = vulkan::VulkanRenderer2D::new(core);

        Self {
            scene: Scene::default(),
            camera_data: camera,
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

    // TODO: error handling
    pub fn draw_frame(&mut self, delta_time: f32) -> bool {
        // TODO: remove
        // std::thread::sleep(std::time::Duration::from_millis(250));
        // let delta_time = 0.1;

        self.scene.update(delta_time);

        let frame_data = &self.renderer_2d.frame_data;
        let curr_frame_idx = self.renderer_2d.curr_frame_idx as u64;

        let camera_buffer = &frame_data.camera_ubos.get(self.renderer_2d.curr_frame_idx).unwrap();
        self.camera_data.update_uniform_buffer(&self.renderer_2d.core, delta_time, camera_buffer).unwrap();

        let scene_ubo = &frame_data.scene_ubo;
        let scene_data = self.scene.get_scene_data_mut();
        scene_data.update_uniform_buffer(&self.renderer_2d.core, scene_ubo, curr_frame_idx).unwrap();

        let object_ubo = &frame_data.object_ubos[curr_frame_idx as usize];
        let render_data = self.scene.get_render_data();
        ObjectData::update_storage_buffer(&self.renderer_2d.core, object_ubo, render_data).unwrap();

        self.renderer_2d.draw_frame(delta_time, render_data)
    }
}
