use ash::vk;
use hell_utils::conversion::c_str_from_char_slice;
use std::ffi::CStr;
use std::fmt;

use crate::vulkan::config;
use crate::vulkan::queues::{self, VulkanQueueSupport};
use crate::vulkan::swapchain::VulkanSwapchain;

use super::surface::VulkanSurface;





pub struct VulkanPhysDevice {
    pub device: vk::PhysicalDevice,
    pub score: u32,
    pub props: vk::PhysicalDeviceProperties,
    pub features: vk::PhysicalDeviceFeatures,
    pub queue_support: VulkanQueueSupport,
}

impl VulkanPhysDevice {
    pub fn pick_phys_device(instance: &ash::Instance, surface: &VulkanSurface) -> Self {
        let all_devices = unsafe { instance.enumerate_physical_devices().unwrap() };

        let device = all_devices
            .into_iter()
            .map(|d| {
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
        surface_data: &VulkanSurface,
        extension_names: &[&str],
    ) -> VulkanPhysDevice {
        let props = unsafe { instance.get_physical_device_properties(phys_device) };
        let features = unsafe { instance.get_physical_device_features(phys_device) };
        let mut score = 0;

        let device_name = unsafe { CStr::from_ptr(props.device_name.as_ptr()) };
        println!("rate device: {:?}", device_name);

        // api version
        // -----------
        let major_version = vk::api_version_major(props.api_version);
        let minor_version = vk::api_version_minor(props.api_version);
        let patch_version = vk::api_version_patch(props.api_version);

        println!(
            "\t> API Version: {}.{}.{}",
            major_version, minor_version, patch_version
        );

        // device-type
        // -----------
        println!("\t> device-type: {:?}", props.device_type);
        match props.device_type {
            vk::PhysicalDeviceType::DISCRETE_GPU => score += 1000,
            _ => score += 100,
        };

        // shaders
        // -------
        // can't function without geo-shaders
        println!(
            "\t> geometry-shader is supported: {:?}",
            features.geometry_shader
        );
        if features.geometry_shader == vk::FALSE {
            score = 0;
        }

        // sampler
        // -------
        println!(
            "\t> sampler-anisotropy is supported: {:?}",
            features.sampler_anisotropy
        );
        if features.sampler_anisotropy == vk::FALSE {
            score = 0;
        }

        // queue-families
        // --------------
        queues::print_queue_families(instance, phys_device);

        let queues = VulkanQueueSupport::new(instance, phys_device, surface_data);
        if !queues.is_complete() {
            score = 0;
            println!("> no suitable queues were found!");
        } else {
            println!("queue-families found: {:?}", queues);
        }

        // extensions
        // ----------
        if !check_device_extension_support(instance, phys_device, extension_names) {
            score = 0;
            println!("> not all device extensions are supported!");
        } else {
            // swap-chains
            // -----------
            let swap_chain_support = VulkanSwapchain::new(phys_device, surface_data);
            if !swap_chain_support.is_suitable() {
                score = 0;
                println!("> no suitable swap-chain found!");
            }
        }

        VulkanPhysDevice {
            device: phys_device,
            score,
            props,
            features,
            queue_support: queues,
        }
    }
}

impl fmt::Debug for VulkanPhysDevice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let device_name = unsafe { CStr::from_ptr(self.props.device_name.as_ptr()) };

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

pub fn find_supported_foramt(
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

