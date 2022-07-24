use winit::window::Window;
use std::os::raw;

pub struct SurfaceInfo {
}

impl From<&Window> for SurfaceInfo {
    fn from(w: &Window) -> Self {



        Self {
            display: x11_display,
            window: x11_window,
        }
    }


}
