use std::os::raw;

use winit::dpi::LogicalSize;
use winit::error::OsError;
use winit::event::{VirtualKeyCode, ElementState, KeyboardInput, WindowEvent, Event};
use winit::event_loop::{EventLoop, ControlFlow};

use crate::utils::fps_limiter::FPSLimiter;




pub struct WinitWindow {
    event_loop: winit::event_loop::EventLoop<()>,
    window: winit::window::Window,
}

impl WinitWindow {
    pub fn new(title: &str, width: u32, height: u32) -> Result<Self, OsError> {
        let event_loop = EventLoop::new();

        let size = LogicalSize::new(width, height);
        let window = winit::window::WindowBuilder::new()
            .with_title(title)
            .with_inner_size(size)
            .build(&event_loop)?;


        Ok(Self {
            event_loop,
            window
        })
    }

    pub fn create_surface_info(&self) -> (*mut raw::c_void, raw::c_ulong) {
        use winit::platform::unix::WindowExtUnix;

        let x11_display = self.window.xlib_display().unwrap();
        let x11_window = self.window.xlib_window().unwrap();

        (x11_display, x11_window)
    }

    pub fn main_loop(self, update_cb: fn(f32) -> ()) {
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
                    // self.draw_frame(delta_time);
                    update_cb(delta_time);

                    fps.tick_frame();
                }
                Event::LoopDestroyed => unsafe {
                    // self.device.device_wait_idle().unwrap();
                },
                _ => {}

            }


        });
    }
}
