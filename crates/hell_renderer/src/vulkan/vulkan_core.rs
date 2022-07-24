use std::ffi::CString;
use std::os::raw;

use ash::prelude::VkResult;
use ash::vk;

use super::surface::{VulkanSurface, VulkanSurfaceCreateInfo};
use super::{validation_layers, platforms, debugging};




const APP_NAME: &str = "hellengine";
const ENGINE_NAME: &str = "hellengine";
const ENGINE_VERSION: u32 = 1;
const API_VERSION: u32 = vk::API_VERSION_1_3;

pub const ENABLE_VALIDATION_LAYERS: bool = true;
pub const VALIDATION_LAYER_NAMES: &[&str] = &[
    "VK_LAYER_KHRONOS_validation"
];




pub struct VulkanCore {
    _entry: ash::Entry,
    instance: ash::Instance,
    // device: ash::Device,
    // phys_device: vk::PhysicalDevice,

    surface: VulkanSurface,
}

impl VulkanCore {
    pub fn new(surface_info: VulkanSurfaceCreateInfo) -> VkResult<Self> {
        let entry = unsafe { ash::Entry::load().unwrap() };
        let instance = create_instance(&entry, APP_NAME)?;
        let surface = VulkanSurface::new(&entry, &instance, surface_info);


        Ok(Self {
            _entry: entry,
            instance,
            surface,
        })
    }
}




fn create_instance(entry: &ash::Entry, app_name: &str) -> VkResult<ash::Instance> {

    if ENABLE_VALIDATION_LAYERS
        && !validation_layers::check_validation_layer_support(entry, VALIDATION_LAYER_NAMES)
    {
        panic!("validation layers requested, but not available!");
    }


    let app_name = CString::new(app_name).unwrap();
    let engine_name = CString::new(ENGINE_NAME).unwrap();

    let app_info = vk::ApplicationInfo::builder()
        .application_name(&app_name)
        .engine_name(&engine_name)
        .engine_version(ENGINE_VERSION)
        .api_version(API_VERSION)
        .build();

    let extension_names = platforms::required_extension_names();

    let mut instance_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_extension_names(&extension_names)
        .build();


    // TODO: improve
    let enabled_validation_layers: Vec<_> = VALIDATION_LAYER_NAMES
        .iter()
        .map(|l| CString::new(*l).unwrap())
        .collect();

    let enabled_validation_layer_ref: Vec<_> = enabled_validation_layers
        .iter()
        .map(|l| l.as_ptr())
        .collect();

    let debug_utils_create_info = debugging::populate_debug_messenger_create_info();

    if ENABLE_VALIDATION_LAYERS {
        instance_info.enabled_layer_count = VALIDATION_LAYER_NAMES.len() as u32;
        instance_info.pp_enabled_layer_names = enabled_validation_layer_ref.as_ptr();
        instance_info.p_next = &debug_utils_create_info
            as *const vk::DebugUtilsMessengerCreateInfoEXT
            as *const raw::c_void;
    }

    unsafe {
        entry.create_instance(&instance_info, None)
    }
}
