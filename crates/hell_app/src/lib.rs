use hell_common::HellResult;
use hell_common::window::{HellWindow, HellWindowExtent};
use hell_renderer::{HellRenderer, HellRendererInfo};
use hell_renderer::vulkan::{ObjectData, config};
use crate::scene::Scene;

mod scene;



pub struct HellApp {
    renderer: HellRenderer,
    scene: Scene,
}


// create
impl HellApp {
    pub fn new(window: &dyn HellWindow) -> HellResult<Self> {
        let surface_info = window.create_surface_info();
        let window_extent = window.get_window_extent();

        let info = HellRendererInfo {
            max_frames_in_flight: config::MAX_FRAMES_IN_FLIGHT, // TODO:
            surface_info,
            window_extent,
        };

        let renderer = HellRenderer::new(info)?;

        Ok(Self {
            renderer,
            scene: Scene::default(),
        })
    }
}

// utils
impl HellApp {
    pub fn handle_window_changed(&mut self, window_extent: HellWindowExtent) {
        self.wait_idle();
        self.renderer.handle_window_changed(window_extent);
    }

    pub fn wait_idle(&self) {
        self.renderer.wait_idle();
    }

    // TODO: error handling
    pub fn draw_frame(&mut self, delta_time: f32) -> HellResult<bool> {
        // TODO: remove
        // std::thread::sleep(std::time::Duration::from_millis(250));
        // let delta_time = 0.1;

        self.scene.update(delta_time);

        let frame_data = self.renderer.get_frame_data();
        let curr_frame_idx = self.renderer.get_frame_idx() as u64;
        let core = self.renderer.get_core();

        let scene_ubo = &frame_data.scene_ubo;
        let scene_data = self.scene.get_scene_data_mut();
        scene_data.update_uniform_buffer(core, scene_ubo, curr_frame_idx).unwrap();

        let object_ubo = &frame_data.object_ubos[curr_frame_idx as usize];
        let render_data = self.scene.get_render_data();
        ObjectData::update_storage_buffer(core, object_ubo, render_data).unwrap();

        self.renderer.draw_frame(delta_time, render_data)
    }
}
