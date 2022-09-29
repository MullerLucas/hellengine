use hell_common::HellResult;
use hell_common::window::{HellWindow, HellWindowExtent};
use hell_renderer::{HellRenderer, HellRendererInfo};
use hell_renderer::vulkan::config;
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

        let scene_data = self.scene.get_scene_data_mut();
        scene_data.update_data();
        self.renderer.update_scene_buffer(scene_data)?;

        let render_data = self.scene.get_render_data();
        self.renderer.update_object_buffer(render_data)?;

        self.renderer.draw_frame(delta_time, render_data)
    }
}
