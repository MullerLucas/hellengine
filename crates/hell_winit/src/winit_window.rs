use hell_app::HellApp;
use hell_common::window::{HellWindow, HellSurfaceInfo, HellWindowExtent};

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
            window,
        })
    }
}

impl HellWindow for WinitWindow {
    fn create_surface_info(&self) -> HellSurfaceInfo {
        use winit::platform::unix::WindowExtUnix;

        let x11_display = self.window.xlib_display().unwrap();
        let x11_window = self.window.xlib_window().unwrap();

        HellSurfaceInfo::new(x11_display, x11_window)
    }

    fn get_window_extent(&self) -> HellWindowExtent {
        let inner_size = self.window.inner_size();

        HellWindowExtent {
            width: inner_size.width,
            height: inner_size.height,
        }
    }
}

impl WinitWindow {
    pub fn get_winit_window_extent(window: &winit::window::Window) -> HellWindowExtent {
        let inner_size = window.inner_size();

        HellWindowExtent {
            width: inner_size.width,
            height: inner_size.height,
        }
    }
}

impl WinitWindow {
    pub fn main_loop(self, mut app: HellApp) {
        let mut fps = FPSLimiter::new();


        let mut handle_resize = false;

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
                    // TODO: check resize logic
                    if handle_resize {
                        let window_extent = WinitWindow::get_winit_window_extent(&self.window);

                        if (window_extent.width * window_extent.height) > 0 {
                            app.on_window_changed(&window_extent);
                            handle_resize = false;
                            println!("> resize was handled...");
                        } else {
                            println!("> can't handle resize - window-extent is zero");
                        }
                    } else {
                        let delta_time = fps.delta_time();
                        handle_resize = app.draw_frame(delta_time);
                    }

                    fps.tick_frame();
                }
                Event::LoopDestroyed => {
                    app.wait_idle();
                },
                _ => {}

            }

            // "drop(app);" on last iteration
        });
    }
}
