// https://raw.githubusercontent.com/unknownue/vulkan-tutorial-rust/master/src/utility/platforms.rs

use ash::extensions;

pub fn required_extension_names() -> Vec<*const i8> {
    vec![
        extensions::khr::Surface::name().as_ptr(),
        extensions::khr::XlibSurface::name().as_ptr(),
        extensions::ext::DebugUtils::name().as_ptr(),   // required for validation layers
    ]
}
