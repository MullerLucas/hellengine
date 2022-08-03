use ash::vk;
use std::os::raw;


pub type VulkanSurfaceCreateInfo = (*mut raw::c_void, raw::c_ulong); //(display, window)

pub struct VulkanSurface {
    pub surface: vk::SurfaceKHR,
    pub surface_loader: ash::extensions::khr::Surface,
}


impl VulkanSurface {
    pub fn new(entry: &ash::Entry, instance: &ash::Instance, surface_info: VulkanSurfaceCreateInfo) -> Self {
        let surface = create_surface(entry, instance, surface_info).unwrap();
        let surface_loader = ash::extensions::khr::Surface::new(entry, instance);

        Self {
            surface,
            surface_loader,
        }
    }
}

// TODO: impl Drop
impl VulkanSurface {
    pub fn drop_manual(&self) {
        println!("> dropping VulkanSurface...");

        unsafe {
            self.surface_loader.destroy_surface(self.surface, None);
        }
    }
}

pub fn create_surface(entry: &ash::Entry, instance: &ash::Instance, surface_info: VulkanSurfaceCreateInfo) -> Result<vk::SurfaceKHR, vk::Result> {
    use std::ptr;

    let x11_display = surface_info.0;
    let x11_window = surface_info.1;

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
