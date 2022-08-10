use std::ffi;
use std::os::raw;

use ash::vk;

use super::{config, platforms, validation_layers, debugging};



pub struct VulkanInstance {
    pub instance: ash::Instance,
    // NOTE: drop after instance has been dropped,
    pub entry: ash::Entry,
}



impl VulkanInstance {
    pub fn new (app_name: &str) -> Self {
        let entry = unsafe { ash::Entry::load().unwrap() };

        if config::ENABLE_VALIDATION_LAYERS
            && !validation_layers::check_validation_layer_support(&entry, config::VALIDATION_LAYER_NAMES)
        {
            panic!("validation layers requested, but not available!");
        }


        let app_name = ffi::CString::new(app_name).unwrap();
        let engine_name = ffi::CString::new(config::ENGINE_NAME).unwrap();

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
            .map(|l| ffi::CString::new(*l).unwrap())
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

        let vk_instance = unsafe { entry.create_instance(&instance_info, None).unwrap() };

        Self {
            entry,
            instance: vk_instance
        }
    }
}

impl Drop for VulkanInstance {
    fn drop(&mut self) {
        println!("> dropping Instance...");

        unsafe {
            self.instance.destroy_instance(None);
        }
    }
}
