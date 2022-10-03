use hell_common::HellResult;
use hell_common::window::{HellWindow, HellWindowExtent};
use hell_renderer::{HellRenderer, HellRendererInfo};
use hell_renderer::vulkan::config;
use hell_resources::ResourceManager;

use crate::scene::Scene;



pub trait HellGame {
    fn init_game(&mut self, scene: &mut Scene, resource_manager: &mut ResourceManager) -> HellResult<()>;
    fn update_game(&mut self, scene: &mut Scene, delta_time: f32) -> HellResult<()>;
}

pub struct HellApp {
    resource_manager: ResourceManager,
    renderer: HellRenderer,
    scene: Option<Scene>,
    // game_info:GameInfo,
    game: &'static mut dyn HellGame,
}


// create
impl<'a> HellApp {
    pub fn new(window: &dyn HellWindow, game: &'static mut dyn HellGame) -> HellResult<Self> {
        let surface_info = window.create_surface_info();
        let window_extent = window.get_window_extent();

        let info = HellRendererInfo {
            max_frames_in_flight: config::MAX_FRAMES_IN_FLIGHT, // TODO:
            surface_info,
            window_extent,
        };

        let resource_manager = ResourceManager::new();
        let renderer = HellRenderer::new(info)?;

        Ok(Self {
            resource_manager,
            renderer,
            scene: None,
            game,
        })
    }

    pub fn create_scene(&self) -> Scene {
        Scene::new()
    }

    // FIX: resource upload
    pub fn load_scene(&'a mut self, scene: Scene) -> HellResult<()> {
        self.scene = Some(scene);
        self.init_game()?;
        self.renderer.upload_resources(&self.resource_manager)?;

        Ok(())
    }
}

impl HellApp {
    pub fn init_game(&mut self) -> HellResult<()> {
        let scene = self.scene.as_mut().unwrap();
        self.game.init_game(scene, &mut self.resource_manager)
    }


    pub fn update_game(&mut self, delta_time: f32) -> HellResult<()> {
        let scene = self.scene.as_mut().unwrap();
        self.game.update_game(scene, delta_time)
    }
}

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


        self.update_game(delta_time)?;

        let scene = match &mut self.scene {
            None => return Ok(false),
            Some(s) => s,
        };

        let scene_data = scene.get_scene_data_mut();
        scene_data.update_data();
        self.renderer.update_scene_buffer(scene_data)?;

        let render_data = scene.get_render_data();
        self.renderer.update_object_buffer(render_data)?;

        self.renderer.draw_frame(delta_time, render_data)
    }
}
