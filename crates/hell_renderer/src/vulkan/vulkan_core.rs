use std::ffi::CString;
use std::os::raw;

use ash::prelude::VkResult;
use ash::vk;

use super::commands::VulkanCommands;
use super::debugging::DebugData;
use super::logic_device::VulkanLogicDevice;
use super::phys_device::VulkanPhysDevice;
use super::surface::{VulkanSurface, VulkanSurfaceCreateInfo};
use super::swapchain::VulkanSwapchain;
use super::{validation_layers, platforms, debugging, config};








pub struct VulkanCore {
    _entry: ash::Entry,
    instance: ash::Instance,
    debug_data: DebugData,

    surface: VulkanSurface,
    phys_device: VulkanPhysDevice,
    device: VulkanLogicDevice,
    swapchain: VulkanSwapchain,
    commands: VulkanCommands,
}

impl VulkanCore {
    pub fn new(surface_info: VulkanSurfaceCreateInfo) -> VkResult<Self> {
        let entry = unsafe { ash::Entry::load().unwrap() };
        let instance = create_instance(&entry, config::APP_NAME)?;
        let debug_data = DebugData::new(&entry, &instance);

        let surface = VulkanSurface::new(&entry, &instance, surface_info);
        let phys_device = VulkanPhysDevice::pick_phys_device(&instance, &surface);
        let device = VulkanLogicDevice::new(&instance, &phys_device, &surface);
        let swapchain = VulkanSwapchain::new(phys_device.device, &surface);
        let commands = VulkanCommands::new(&device);



        Ok(Self {
            _entry: entry,
            instance,
            surface,
            phys_device,
            device,
            swapchain,
            commands,

            debug_data,
        })
    }
}

impl Drop for VulkanCore {
    // TODO:
    fn drop(&mut self) {

    }
}




fn create_instance(entry: &ash::Entry, app_name: &str) -> VkResult<ash::Instance> {

    if config::ENABLE_VALIDATION_LAYERS
        && !validation_layers::check_validation_layer_support(entry, config::VALIDATION_LAYER_NAMES)
    {
        panic!("validation layers requested, but not available!");
    }


    let app_name = CString::new(app_name).unwrap();
    let engine_name = CString::new(config::ENGINE_NAME).unwrap();

    let app_info = vk::ApplicationInfo::builder()
        .application_name(&app_name)
        .engine_name(&engine_name)
        .engine_version(config::ENGINE_VERSION)
        .api_version(config::API_VERSION)
        .build();

    let extension_names = platforms::required_extension_names();

    let mut instance_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_extension_names(&extension_names)
        .build();


    // TODO: improve
    let enabled_validation_layers: Vec<_> = config::VALIDATION_LAYER_NAMES
        .iter()
        .map(|l| CString::new(*l).unwrap())
        .collect();

    let enabled_validation_layer_ref: Vec<_> = enabled_validation_layers
        .iter()
        .map(|l| l.as_ptr())
        .collect();

    let debug_utils_create_info = debugging::populate_debug_messenger_create_info();

    if config::ENABLE_VALIDATION_LAYERS {
        instance_info.enabled_layer_count = config::VALIDATION_LAYER_NAMES.len() as u32;
        instance_info.pp_enabled_layer_names = enabled_validation_layer_ref.as_ptr();
        instance_info.p_next = &debug_utils_create_info
            as *const vk::DebugUtilsMessengerCreateInfoEXT
            as *const raw::c_void;
    }

    unsafe {
        entry.create_instance(&instance_info, None)
    }
}

