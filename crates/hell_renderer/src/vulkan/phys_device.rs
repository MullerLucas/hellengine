use ash::vk;
use hell_utils::conversion::c_str_from_char_slice;
use std::ffi::CStr;
use std::fmt;

use crate::vulkan::config;
use crate::vulkan::queues::{self, VulkanQueueSupport};

use super::surface::VulkanSurface;
use super::swapchain::VulkanSwapchainSupport;

pub struct VulkanPhysDevice {
    pub device: vk::PhysicalDevice,
    pub score: u32,
    pub device_props: vk::PhysicalDeviceProperties,
    pub features: vk::PhysicalDeviceFeatures,
    pub queue_support: VulkanQueueSupport,
    pub swapchain_support: VulkanSwapchainSupport,
    pub depth_format: vk::Format,
}

impl VulkanPhysDevice {
    pub fn pick_phys_device(instance: &ash::Instance, surface: &VulkanSurface) -> Self {
        let all_devices = unsafe { instance.enumerate_physical_devices().unwrap() };

        let device = all_devices
            .into_iter()
            .flat_map(|d| {
                VulkanPhysDevice::rate_device_suitability(
                    instance,
                    d,
                    surface,
                    config::DEVICE_EXTENSION_NAMES,
                )
            })
            .filter(|d| d.score > 0)
            .max_by(|r1, r2| r1.score.cmp(&r2.score));

        let device = match device {
            None => {
                panic!("no suitable physical device found");
            }
            Some(d) => d,
        };

        println!("physical device picked: {:?}", device);

        device
    }

    pub fn rate_device_suitability(
        instance: &ash::Instance,
        phys_device: vk::PhysicalDevice,
        surface: &VulkanSurface,
        extension_names: &[&str],
    ) -> Option<VulkanPhysDevice> {
        let device_props = unsafe { instance.get_physical_device_properties(phys_device) };
        let features = unsafe { instance.get_physical_device_features(phys_device) };
        let mut _score = 0;

        let device_name = unsafe { CStr::from_ptr(device_props.device_name.as_ptr()) };
        println!("rate device: {:?}", device_name);

        // api version
        // -----------
        let major_version = vk::api_version_major(device_props.api_version);
        let minor_version = vk::api_version_minor(device_props.api_version);
        let patch_version = vk::api_version_patch(device_props.api_version);

        println!(
            "\t> API Version: {}.{}.{}",
            major_version, minor_version, patch_version
        );

        // device-type
        // -----------
        println!("\t> device-type: {:?}", device_props.device_type);
        match device_props.device_type {
            vk::PhysicalDeviceType::DISCRETE_GPU => _score += 1000,
            _ => _score += 100,
        };

        // shaders
        // -------
        // can't function without geo-shaders
        println!(
            "\t> geometry-shader is supported: {:?}",
            features.geometry_shader
        );
        if features.geometry_shader == vk::FALSE {
            _score = 0;
        }

        // sampler
        // -------
        println!(
            "\t> sampler-anisotropy is supported: {:?}",
            features.sampler_anisotropy
        );
        if features.sampler_anisotropy == vk::FALSE {
            _score = 0;
        }

        // queue-families
        // --------------
        queues::print_queue_families(instance, phys_device);

        let queue_support = VulkanQueueSupport::new(instance, phys_device, surface);
        if !queue_support.is_complete() {
            _score = 0;
            println!("> no suitable queues were found!");
        } else {
            println!("queue-families found: {:?}", queue_support);
        }

        // extensions
        // ----------
        let swapchain_support =
            if !check_device_extension_support(instance, phys_device, extension_names) {
                _score = 0;
                println!("> not all device extensions are supported!");
                return None;
            } else {
                // swap-chains
                // -----------
                let swapchain_support = VulkanSwapchainSupport::new(phys_device, surface);
                if !swapchain_support.is_suitable() {
                    _score = 0;
                    println!("> no suitable swap-chain found!");
                }
                swapchain_support
            };

        let depth_format = find_depth_format(instance, phys_device);

        Some(VulkanPhysDevice {
            device: phys_device,
            score: _score,
            device_props,
            features,
            queue_support,
            swapchain_support,
            depth_format,
        })
    }
}

impl fmt::Debug for VulkanPhysDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let device_name = unsafe { CStr::from_ptr(self.device_props.device_name.as_ptr()) };

        write!(
            f,
            "DeviceSuitability: '{:?}'::'{:?}' => {}",
            device_name, self.device, self.score
        )
    }
}

fn check_device_extension_support(
    instance: &ash::Instance,
    phys_device: vk::PhysicalDevice,
    extension_names: &[&str],
) -> bool {
    let extension_props = unsafe {
        instance
            .enumerate_device_extension_properties(phys_device)
            .unwrap()
    };
    let mut remaining_extensions = extension_names.to_owned();

    println!("checking extension support...");
    println!("\t> supported extensions: ");
    println!("\t\thidden");

    for prop in extension_props {
        let ext = c_str_from_char_slice(&prop.extension_name)
            .to_str()
            .unwrap();
        // println!("\t\t> {:?}", ext);

        remaining_extensions.retain(|e| *e != ext);
    }

    println!("\t> un-supported extensions: ");
    for ext in &remaining_extensions {
        println!("\t\t> {:?}", ext);
    }

    remaining_extensions.is_empty()
}

pub fn find_supported_format(
    instance: &ash::Instance,
    phys_device: vk::PhysicalDevice,
    candidates: &[vk::Format],
    tiling: vk::ImageTiling,
    features: vk::FormatFeatureFlags,
) -> vk::Format {
    for format in candidates {
        let props = unsafe { instance.get_physical_device_format_properties(phys_device, *format) };

        match tiling {
            vk::ImageTiling::LINEAR => {
                if (props.linear_tiling_features & features) == features {
                    return *format;
                };
            }
            vk::ImageTiling::OPTIMAL => {
                if (props.optimal_tiling_features & features) == features {
                    return *format;
                };
            }
            _ => {}
        }
    }

    panic!("failed to find supported format!");
}

fn find_depth_format(instance: &ash::Instance, phys_device: vk::PhysicalDevice) -> vk::Format {
    find_supported_format(
        instance,
        phys_device,
        &[
            vk::Format::D32_SFLOAT,
            vk::Format::D32_SFLOAT_S8_UINT,
            vk::Format::D24_UNORM_S8_UINT
        ],
        vk::ImageTiling::OPTIMAL,
        vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT
    )
}


pub fn get_max_useable_sample_count(
    instance: &ash::Instance,
    phys_device: vk::PhysicalDevice,
) -> vk::SampleCountFlags {
    let phys_device_props = unsafe { instance.get_physical_device_properties(phys_device) };

    let counts = phys_device_props.limits.framebuffer_color_sample_counts
        & phys_device_props.limits.framebuffer_depth_sample_counts;

    if counts.contains(vk::SampleCountFlags::TYPE_64) {
        vk::SampleCountFlags::TYPE_64
    } else if counts.contains(vk::SampleCountFlags::TYPE_32) {
        vk::SampleCountFlags::TYPE_32
    } else if counts.contains(vk::SampleCountFlags::TYPE_16) {
        vk::SampleCountFlags::TYPE_16
    } else if counts.contains(vk::SampleCountFlags::TYPE_8) {
        vk::SampleCountFlags::TYPE_8
    } else if counts.contains(vk::SampleCountFlags::TYPE_4) {
        vk::SampleCountFlags::TYPE_4
    } else if counts.contains(vk::SampleCountFlags::TYPE_2) {
        vk::SampleCountFlags::TYPE_2
    } else {
        vk::SampleCountFlags::TYPE_1
    }
}
