use hell_app::HellApp;
use hell_common::window::{HellWindow, HellSurfaceInfo, HellWindowExtent};

use hell_core::error::{HellResult, OptToHellErr};
use winit::dpi::LogicalSize;
use winit::error::OsError;
use winit::event::{VirtualKeyCode, ElementState, KeyboardInput, WindowEvent, Event};
use winit::event_loop::{EventLoop, ControlFlow};

use crate::input::{keycode_to_hell, element_state_to_hell};
use crate::utils::fps_limiter::FPSLimiter;




pub struct Window {
    event_loop: winit::event_loop::EventLoop<()>,
    window: winit::window::Window,
}

impl Window {
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

impl HellWindow for Window {
    fn create_surface_info(&self) -> HellResult<HellSurfaceInfo> {
        use winit::platform::unix::WindowExtUnix;

        let x11_display = self.window.xlib_display().to_generic_hell_err()?;
        let x11_window = self.window.xlib_window().to_window_hell_err()?;

        Ok(HellSurfaceInfo::new(x11_display, x11_window))
    }

    fn get_window_extent(&self) -> HellWindowExtent {
        let inner_size = self.window.inner_size();

        HellWindowExtent {
            width: inner_size.width,
            height: inner_size.height,
        }
    }
}

impl Window {
    pub fn get_winit_window_extent(window: &winit::window::Window) -> HellWindowExtent {
        let inner_size = window.inner_size();

        HellWindowExtent {
            width: inner_size.width,
            height: inner_size.height,
        }
    }
}

impl Window {
    // TODO: error handling
    pub fn main_loop(self, mut app: HellApp) {
        let mut fps = FPSLimiter::new();
        let mut handle_resize = false;


        self.event_loop.run(move |event, _, control_flow| {
            // continously run the event loop, event if the OS hasn't dispatched any events
            control_flow.set_poll();
            // wait until events are available
            // control_flow.set_wait();

            app.advance_frame().unwrap();

            match event {
                Event::WindowEvent { event, .. } => {
                    Self::handle_window_event(&event, control_flow, &mut app).expect("failed to handle window event");
                },
                Event::MainEventsCleared => {
                    self.window.request_redraw();
                }
                Event::RedrawRequested(_) => {
                    Window::handle_redraw_request(&mut handle_resize, &self.window, &mut app, &mut fps).expect("failed to handle redraw request");
                }
                Event::LoopDestroyed => {
                    app.wait_idle().expect("failed to wait for the app to become idle");
                },
                _ => {}

            }

            // "drop(app);" on last iteration
        });
    }

    fn handle_window_event(event: &winit::event::WindowEvent, control_flow: &mut winit::event_loop::ControlFlow, app: &mut HellApp) -> HellResult<()> {
        match event {
            WindowEvent::CloseRequested => { *control_flow = ControlFlow::Exit },

            WindowEvent::KeyboardInput { input: KeyboardInput { virtual_keycode: Some(VirtualKeyCode::Escape), state: ElementState::Pressed, .. }, .. } => {
                println!("> window-event: escape pressed");
                *control_flow = ControlFlow::Exit;
            }

            WindowEvent::KeyboardInput { input: KeyboardInput { virtual_keycode: Some(keycode), state, .. }, .. } => {
                let keycode = keycode_to_hell(keycode);
                let state = element_state_to_hell(*state);
                app.input.update_key_state(keycode, state)?;
            }

            _ => (),
        };

        Ok(())
    }

    fn handle_redraw_request(handle_resize: &mut bool, window: &winit::window::Window, app: &mut HellApp, fps: &mut FPSLimiter) -> HellResult<()> {
        // TODO: check resize logic
        if *handle_resize {
            let window_extent = Window::get_winit_window_extent(window);

            if (window_extent.width * window_extent.height) > 0 {
                app.handle_window_changed(window_extent)?;
                *handle_resize = false;
                println!("> resize was handled...");
            } else {
                println!("> can't handle resize - window-extent is zero");
            }
        } else {
            let delta_time = fps.delta_time();
            *handle_resize = app.draw_frame(delta_time)?;
        }

        fps.tick_frame();

        Ok(())
    }
}
