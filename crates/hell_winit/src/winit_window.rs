use std::os::raw;

use hell_renderer::vulkan::pipeline::VulkanGraphicsPipeline;
use hell_renderer::vulkan::vulkan_core::VulkanCore;

use winit::dpi::LogicalSize;
use winit::error::OsError;
use winit::event::{VirtualKeyCode, ElementState, KeyboardInput, WindowEvent, Event};
use winit::event_loop::{EventLoop, ControlFlow};
use winit::window::Window;

use crate::utils::fps_limiter::FPSLimiter;




pub struct WinitWindow {
    event_loop: winit::event_loop::EventLoop<()>,
    window: winit::window::Window,

    // TODO: restructure
    core: VulkanCore,
    pipeline: VulkanGraphicsPipeline,
}

impl WinitWindow {
    pub fn new(title: &str, width: u32, height: u32) -> Result<Self, OsError> {
        let event_loop = EventLoop::new();

        let size = LogicalSize::new(width, height);
        let window = winit::window::WindowBuilder::new()
            .with_title(title)
            .with_inner_size(size)
            .build(&event_loop)?;

        let core = VulkanCore::new(WinitWindow::create_surface_info(&window)).unwrap();
        let pipeline = VulkanGraphicsPipeline::new(&core);

        Ok(Self {
            event_loop,
            window,

            core,
            pipeline,
        })
    }
}

// TODO: impl Drop
// impl WinitWindow {
//     fn drop_manual(&self) {
//         println!("> dropping WinitWindow...");
//
//         self.pipeline.drop_manual(&self.core.device.device);
//     }
// }


impl WinitWindow {

    pub fn create_surface_info(window: &Window) -> (*mut raw::c_void, raw::c_ulong) {
        use winit::platform::unix::WindowExtUnix;

        let x11_display = window.xlib_display().unwrap();
        let x11_window = window.xlib_window().unwrap();

        (x11_display, x11_window)
    }

    pub fn main_loop(mut self) {
        let mut fps = FPSLimiter::new();

        self.event_loop.run(move |event, _, control_flow| {

            match event {
                Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    } => {
                        dbg!("> escape pressed");
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => (),
                },
                Event::MainEventsCleared => {
                    self.window.request_redraw();
                }
                Event::RedrawRequested(_) => {
                    let delta_time = fps.delta_time();
                    self.core.draw_frame(&self.pipeline, delta_time);

                    fps.tick_frame();
                }
                Event::LoopDestroyed => {
                    self.core.device.wait_idle();
                    // TODO: drop
                    // self.drop_manual();
                    // self.pipeline.drop_manual(&self.core.device.device);
                },
                _ => {}

            }
        });

    }
}
