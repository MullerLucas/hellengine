use hell_common::window::{HellWindow, HellWindowExtent};
use hell_core::config;
use hell_error::HellResult;
use hell_input::InputManager;
use hell_renderer::render_types::RenderPackage;
use hell_renderer::shader::SpriteShaderSceneData;
use hell_renderer::{HellRenderer, HellRendererInfo};
use hell_resources::ResourceManager;




// ----------------------------------------------------------------------------
// hell-game
// ----------------------------------------------------------------------------

pub trait HellGame {
    fn scene_data(&self) -> &SpriteShaderSceneData;
    fn scene_data_mut(&mut self) -> &mut SpriteShaderSceneData;
    fn render_package(&self) -> &RenderPackage;

    fn init_game(&mut self, resource_manager: &mut ResourceManager) -> HellResult<()>;
    fn update_game(&mut self, delta_time: f32, input: &InputManager) -> HellResult<()>;
}



// ----------------------------------------------------------------------------
// hell-app
// ----------------------------------------------------------------------------

pub struct HellApp {
    resource_manager: ResourceManager,
    renderer: HellRenderer,
    game: &'static mut dyn HellGame,
    pub input: InputManager,
}


// create
impl HellApp {
    pub fn new(window: &dyn HellWindow, game: &'static mut dyn HellGame) -> HellResult<Self> {
        let surface_info = window.create_surface_info()?;
        let window_extent = window.get_window_extent();

        let info = HellRendererInfo {
            max_frames_in_flight: config::FRAMES_IN_FLIGHT,
            surface_info,
            window_extent,
        };

        let resource_manager = ResourceManager::new();
        let renderer = HellRenderer::new(info)?;
        let input = InputManager::new();

        Ok(Self {
            resource_manager,
            renderer,
            game,
            input,
        })
    }
}

impl HellApp {
    pub fn init_game(&mut self) -> HellResult<()> {
        self.game.init_game(&mut self.resource_manager)?;

        self.resource_manager.load_used_textures()?;
        self.renderer.prepare_renderer(&self.resource_manager)?;

        Ok(())
    }


    fn update_game(&mut self, delta_time: f32) -> HellResult<()> {
        self.game.update_game(delta_time, &self.input)
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

        let scene_data = self.game.scene_data();
        let render_pkg = self.game.render_package();

        self.renderer.draw_frame(delta_time, scene_data, render_pkg, &self.resource_manager)
    }
}
