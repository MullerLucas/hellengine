use ash::vk;
use std::os::raw;
use std::{ptr, ffi};





pub struct VulkanDebugData {
    pub debug_utils_loader: ash::extensions::ext::DebugUtils,
    pub debug_messenger: vk::DebugUtilsMessengerEXT,
}

impl VulkanDebugData {
    pub fn new(entry: &ash::Entry, instance: &ash::Instance) -> Self {
        let debug_utils_loader = ash::extensions::ext::DebugUtils::new(entry, instance);
        let messenger_create_info = populate_debug_messenger_create_info();
        let debug_messenger = unsafe { debug_utils_loader.create_debug_utils_messenger(&messenger_create_info, None).expect("failed to create debug_utils_messenger") };

        Self {
            debug_utils_loader,
            debug_messenger,
        }
    }
}

impl Drop for VulkanDebugData {
    fn drop(&mut self) {
        unsafe {
            println!("> dropping DebugData");

            self.debug_utils_loader.destroy_debug_utils_messenger(self.debug_messenger, None);
        }
    }
}



pub fn populate_debug_messenger_create_info() -> vk::DebugUtilsMessengerCreateInfoEXT {
    vk::DebugUtilsMessengerCreateInfoEXT {
        s_type: vk::StructureType::DEBUG_UTILS_MESSENGER_CREATE_INFO_EXT,
        p_next: ptr::null(),
        flags: vk::DebugUtilsMessengerCreateFlagsEXT::empty(),
        message_severity:
        // vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE |
        // vk::DebugUtilsMessageSeverityFlagsEXT::INFO |
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING |
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
        message_type:
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL |
            vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE |
            vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
        pfn_user_callback: Some(vulkan_debug_utils_callback),
        p_user_data: ptr::null_mut()
    }
}


unsafe extern "system" fn vulkan_debug_utils_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut raw::c_void,
) -> vk::Bool32 {

    let severity = match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => "[VERBOSE]",
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => "[WARNING]",
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => "[ERROR  ]",
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => "[INFO   ]",
        _ => "[UNKNOWN]",
    };

    let types = match message_type {
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => "[GENERAL    ]",
        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "[PERFORMANCE]",
        vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "[VALIDATION ]",
        _ => "[UNKNOWN    ]",
    };

    let message = ffi::CStr::from_ptr((*p_callback_data).p_message);

    println!("[DEBUG]{severity}{types}{message:?}");

    if message_severity == vk::DebugUtilsMessageSeverityFlagsEXT::ERROR ||
       message_severity == vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
    {
        panic!("THERE WAS AN VK-ERROR!");
    }

    // vulkan call that triggered validation-layer messsage should be aborted?
    vk::FALSE
}
