use hell_common::window::{HellWindow, HellWindowExtent};
use hell_core::error::HellResult;
use hell_input::InputManager;
use hell_renderer::render_types::RenderPackage;
use hell_renderer::{HellRenderer, HellRendererInfo, config};




// ----------------------------------------------------------------------------
// hell-game
// ----------------------------------------------------------------------------

pub trait HellGame {
    fn render_package(&self) -> &RenderPackage;

    fn init_game(&mut self, renderer: &mut HellRenderer) -> HellResult<()>;
    fn update_game(&mut self, delta_time: f32, input: &InputManager) -> HellResult<()>;
}



// ----------------------------------------------------------------------------
// hell-app
// ----------------------------------------------------------------------------

pub struct HellApp {
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

        let renderer = HellRenderer::new(info)?;
        let input = InputManager::new();

        Ok(Self {
            renderer,
            game,
            input,
        })
    }
}

impl HellApp {
    pub fn init_game(&mut self) -> HellResult<()> {
        self.game.init_game(&mut self.renderer)?;
        self.renderer.prepare_renderer()?;

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
        let render_pkg = self.game.render_package();
        self.renderer.draw_frame(delta_time, render_pkg)
    }
}
