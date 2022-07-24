// https://raw.githubusercontent.com/unknownue/vulkan-tutorial-rust/master/src/utility/platforms.rs

use std::os::raw;

// use ash::version::{EntryV1_0, InstanceV1_0};
use ash::{Entry, Instance};
use ash::vk;

#[cfg(target_os = "windows")]
use ash::extensions::khr::Win32Surface;
// #[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
// use ash::extensions::khr::XlibSurface;
#[cfg(target_os = "macos")]
use ash::extensions::mvk::MacOSSurface;

use ash::extensions;

#[cfg(target_os = "macos")]
use cocoa::appkit::{NSView, NSWindow};
#[cfg(target_os = "macos")]
use cocoa::base::id as cocoa_id;
#[cfg(target_os = "macos")]
use metal::CoreAnimationLayer;
#[cfg(target_os = "macos")]
use objc::runtime::YES;

// required extension ------------------------------------------------------
#[cfg(target_os = "macos")]
pub fn required_extension_names() -> Vec<*const i8> {
    vec![
        Surface::name().as_ptr(),
        MacOSSurface::name().as_ptr(),
        DebugUtils::name().as_ptr(),
    ]
}

#[cfg(all(windows))]
pub fn required_extension_names() -> Vec<*const i8> {
    vec![
        Surface::name().as_ptr(),
        Win32Surface::name().as_ptr(),
        DebugUtils::name().as_ptr(),
    ]
}

#[cfg(all(unix, not(target_os = "android"), not(target_os = "macos")))]
pub fn required_extension_names() -> Vec<*const i8> {
    vec![
        extensions::khr::Surface::name().as_ptr(),
        extensions::khr::XlibSurface::name().as_ptr(),
        extensions::ext::DebugUtils::name().as_ptr(),   // required for validation layers
    ]
}
// ------------------------------------------------------------------------

// create surface ---------------------------------------------------------

