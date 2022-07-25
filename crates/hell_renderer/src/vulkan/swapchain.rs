use ash::vk;

use super::surface::VulkanSurface;


pub struct VulkanSwapchain {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>
}

impl VulkanSwapchain {
    pub fn new(phys_device: vk::PhysicalDevice, surface_data: &VulkanSurface) -> Self {
        let capabilities = unsafe {
            surface_data
                .surface_loader
                .get_physical_device_surface_capabilities(phys_device, surface_data.surface)
                .unwrap()
        };
        let formats = unsafe {
            surface_data
                .surface_loader
                .get_physical_device_surface_formats(phys_device, surface_data.surface)
                .unwrap()
        };
        let present_modes = unsafe {
            surface_data
                .surface_loader
                .get_physical_device_surface_present_modes(phys_device, surface_data.surface)
                .unwrap()
        };



        Self {
            capabilities,
            formats,
            present_modes,
        }
    }

    pub fn is_suitable(&self) -> bool {
        !self.formats.is_empty() && !self.present_modes.is_empty()
    }

    pub fn choose_swap_surface_format(&self) -> vk::SurfaceFormatKHR {
        let desired_format = self.formats.iter()
            .find(|f| {
                f.format == vk::Format::B8G8R8_SRGB && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            });

        if let Some(desired_format) = desired_format {
            *desired_format
        } else {
            *self.formats.first().unwrap()
        }
    }

    pub fn choose_swap_present_mode(&self) -> vk::PresentModeKHR {
        if let Some(desired_mode) = self
            .present_modes
            .iter()
            .find(|m| **m == vk::PresentModeKHR::MAILBOX)
        {
            *desired_mode
        } else {
            // guaranteed to be available
            vk::PresentModeKHR::FIFO
        }
    }

    pub fn choose_swap_extend(&self, win_width: u32, win_height: u32) -> vk::Extent2D {
        if self.capabilities.current_extent.width != u32::MAX {
            self.capabilities.current_extent
        } else {
            vk::Extent2D {
                width: num::clamp(
                    win_width,
                    self.capabilities.min_image_extent.width,
                    self.capabilities.max_image_extent.width,
                ),
                height: num::clamp(
                    win_height,
                    self.capabilities.min_image_extent.height,
                    self.capabilities.max_image_extent.height,
                ),
            }
        }
    }

    pub fn choose_img_count(&self) -> u32 {
        let desired_img_count = self.capabilities.min_image_count + 1;

        // if there is no limit
        if self.capabilities.max_image_count == 0 {
            desired_img_count
        } else {
            desired_img_count.min(self.capabilities.max_image_count)
        }
    }
}
