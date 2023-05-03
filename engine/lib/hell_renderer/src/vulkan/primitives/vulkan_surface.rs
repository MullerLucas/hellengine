use ash::vk;
use hell_common::window::HellSurfaceInfo;
use hell_core::error::{HellResult, ErrToHellErr};


pub struct VulkanSurface {
    pub surface: vk::SurfaceKHR,
    pub surface_loader: ash::extensions::khr::Surface,
}


impl VulkanSurface {
    pub fn new(entry: &ash::Entry, instance: &ash::Instance, surface_info: &HellSurfaceInfo) -> HellResult<Self> {
        let surface = create_surface(entry, instance, surface_info).to_render_hell_err()?;
        let surface_loader = ash::extensions::khr::Surface::new(entry, instance);

        Ok(Self {
            surface,
            surface_loader,
        })
    }
}

impl Drop for VulkanSurface {
    fn drop(&mut self) {
        println!("> dropping Surface...");

        unsafe {
            self.surface_loader.destroy_surface(self.surface, None);
        }
    }
}

pub fn create_surface(entry: &ash::Entry, instance: &ash::Instance, surface_info: &HellSurfaceInfo) -> Result<vk::SurfaceKHR, vk::Result> {
    use std::ptr;

    let x11_display = surface_info.get_display();
    let x11_window = surface_info.get_window();

    let x11_create_info = vk::XlibSurfaceCreateInfoKHR {
        s_type: vk::StructureType::XLIB_SURFACE_CREATE_INFO_KHR,
        p_next: ptr::null(),
        flags: Default::default(),
        window: x11_window as vk::Window,
        dpy: x11_display as *mut vk::Display,
    };

    let xlib_surface_loader = ash::extensions::khr::XlibSurface::new(entry, instance);

    unsafe {
        xlib_surface_loader.create_xlib_surface(&x11_create_info, None)
    }
}
