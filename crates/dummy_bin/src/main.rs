use hell_renderer::vulkan::pipeline::VulkanGraphicsPipeline;
use hell_renderer::vulkan::vulkan_core::VulkanCore;

// TODO
static mut tmp_app: Option<TmpApp> = None;

fn main() {
    let win = hell_winit::WinitWindow::new("hell_app", 800, 600)
        .expect("failed to create window");

    unsafe {
        tmp_app = Some(TmpApp::new(&win));
    }
    win.main_loop(update);




}
fn update(delta_time: f32) {
    let mut app = unsafe { tmp_app.unwrap() };
    app.core.draw_frame(&app.pipeline, delta_time);
}

struct TmpApp {
    pub core: VulkanCore,
    pub pipeline: VulkanGraphicsPipeline
}

impl TmpApp {
    pub fn new(win: &hell_winit::WinitWindow) -> Self {
        let core = VulkanCore::new(win.create_surface_info()).unwrap();
        let pipeline = VulkanGraphicsPipeline::new(&core);


        Self { core, pipeline }
    }
}

