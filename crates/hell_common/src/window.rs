use std::os::raw;


pub struct HellSurfaceInfo {
    display: *mut raw::c_void,
    window: raw::c_ulong,
}

impl HellSurfaceInfo {
    pub fn new(display: *mut raw::c_void, window: raw::c_ulong) -> Self {
        Self { display, window }
    }

    pub const fn get_display(&self) -> *mut raw::c_void {
        self.display
    }

    pub const fn get_window(&self) -> raw::c_ulong {
        self.window
    }
}




pub struct HellWindowExtent {
    pub width: u32,
    pub height: u32,
}




pub trait HellWindow {
    fn create_surface_info(&self) -> HellSurfaceInfo;
    fn get_window_extent(&self) -> HellWindowExtent;
}
