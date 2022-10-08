use hell_common::window::{HellWindow, HellWindowExtent};
use hell_error::{HellResult, OptToHellErr};
use hell_input::InputManager;
use hell_renderer::{HellRenderer, HellRendererInfo};
use hell_renderer::vulkan::config;
use hell_resources::ResourceManager;

use crate::scene::Scene;


// ----------------------------------------------------------------------------
// hell-game
// ----------------------------------------------------------------------------

pub trait HellGame {
    fn init_game(&mut self, scene: &mut Scene, resource_manager: &mut ResourceManager) -> HellResult<()>;
    fn update_game(&mut self, scene: &mut Scene, input: &InputManager, delta_time: f32) -> HellResult<()>;
}



// ----------------------------------------------------------------------------
// hell-app
// ----------------------------------------------------------------------------

pub struct HellApp {
    resource_manager: ResourceManager,
    renderer: HellRenderer,
    scene: Option<Scene>,
    // game_info:GameInfo,
    game: &'static mut dyn HellGame,
    pub input: InputManager,
}


// create
impl<'a> HellApp {
    pub fn new(window: &dyn HellWindow, game: &'static mut dyn HellGame) -> HellResult<Self> {
        let surface_info = window.create_surface_info()?;
        let window_extent = window.get_window_extent();

        let info = HellRendererInfo {
            max_frames_in_flight: config::MAX_FRAMES_IN_FLIGHT,
            surface_info,
            window_extent,
        };

        let resource_manager = ResourceManager::new();
        let renderer = HellRenderer::new(info)?;
        let input = InputManager::new();

        Ok(Self {
            resource_manager,
            renderer,
            scene: None,
            game,
            input,
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
        let scene = self.scene.as_mut().to_render_hell_err()?;
        self.game.init_game(scene, &mut self.resource_manager)
    }


    pub fn update_game(&mut self, delta_time: f32) -> HellResult<()> {
        let scene = self.scene.as_mut().to_render_hell_err()?;
        self.game.update_game(scene, &self.input, delta_time)
    }
}

impl HellApp {
    pub fn handle_window_changed(&mut self, window_extent: HellWindowExtent) -> HellResult<()> {
        self.wait_idle()?;
        self.renderer.handle_window_changed(window_extent)
    }

    pub fn wait_idle(&self) -> HellResult<()> {
        self.renderer.wait_idle()
    }

    pub fn advance_frame(&mut self) -> HellResult<()> {
        self.input.reset_released_keys();

        Ok(())
    }

    pub fn draw_frame(&mut self, delta_time: f32) -> HellResult<bool> {
        // std::thread::sleep(std::time::Duration::from_millis(250));
        // let delta_time = 0.1;

        self.update_game(delta_time)?;

        let scene = match &mut self.scene {
            None => return Ok(false),
            Some(s) => s,
        };

        let scene_data = scene.get_scene_data_mut();
        // scene_data.update_data()?;
        self.renderer.update_scene_buffer(scene_data)?;

        let render_data = scene.get_render_data();
        self.renderer.update_object_buffer(render_data)?;

        self.renderer.draw_frame(delta_time, render_data)
    }
}
