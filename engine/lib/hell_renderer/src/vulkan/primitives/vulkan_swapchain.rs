use ash::vk;
use hell_common::window::HellWindowExtent;
use hell_core::error::{HellResult, OptToHellErr};
use crate::{vulkan::{primitives::VulkanImage,  VulkanContextRef}, config};

use super::{VulkanSemaphore, VulkanSurface};





// ----------------------------------------------------------------------------
// swapchain support
// ----------------------------------------------------------------------------

pub struct VulkanSwapchainSupport {
    pub capabilities: vk::SurfaceCapabilitiesKHR,
    pub formats: Vec<vk::SurfaceFormatKHR>,
    pub present_modes: Vec<vk::PresentModeKHR>
}

impl VulkanSwapchainSupport {
    pub fn new(phys_device: vk::PhysicalDevice, surface: &VulkanSurface) -> HellResult<Self> {
        let capabilities = unsafe {
            surface
                .surface_loader
                .get_physical_device_surface_capabilities(phys_device, surface.surface)?
        };
        let formats = unsafe {
            surface
                .surface_loader
                .get_physical_device_surface_formats(phys_device, surface.surface)?
        };
        let present_modes = unsafe {
            surface
                .surface_loader
                .get_physical_device_surface_present_modes(phys_device, surface.surface)?
        };



        Ok(Self {
            capabilities,
            formats,
            present_modes,
        })
    }

    pub fn is_suitable(&self) -> bool {
        !self.formats.is_empty() && !self.present_modes.is_empty()
    }

    pub fn choose_swap_surface_format(&self) -> HellResult<vk::SurfaceFormatKHR> {
        let desired_format = self.formats.iter()
            .find(|f| {
                f.format == vk::Format::B8G8R8A8_SRGB && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            });

        if let Some(desired_format) = desired_format {
            Ok(*desired_format)
        } else {
            Ok(*self.formats.first().to_render_hell_err()?)
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
            config::FALLBACK_PRESENT_MODE
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




// ----------------------------------------------------------------------------
// swapchain support
// ----------------------------------------------------------------------------

pub struct VulkanSwapchain {
    ctx: VulkanContextRef,

    pub vk_swapchain: vk::SwapchainKHR,
    pub swapchain_loader: ash::extensions::khr::Swapchain,
    pub swapchain_support: VulkanSwapchainSupport,

    pub imgs: Vec<vk::Image>,
    pub views: Vec<vk::ImageView>,

    pub surface_format: vk::SurfaceFormatKHR,
    pub extent: vk::Extent2D,

    // TODO: remove useless array format
    pub viewport: [vk::Viewport; 1],
    pub sissor: [vk::Rect2D; 1],

    was_dropped: bool,
}

impl Drop for VulkanSwapchain {
    fn drop(&mut self) {
        if !self.was_dropped {
            self.drop_manual();
        }
    }
}

impl VulkanSwapchain {
    pub fn drop_manual(&mut self) {
        println!("> dropping Swapchain...");

        unsafe {
            let device = &self.ctx.device.handle;
            self.views.iter().for_each(|&v| {
                device.destroy_image_view(v, None);
            });
            // cleans up all swapchain images
            self.swapchain_loader.destroy_swapchain(self.vk_swapchain, None);
        }

        self.was_dropped = true;
    }
}

impl VulkanSwapchain {
    pub fn new(ctx: &VulkanContextRef, window_extent: HellWindowExtent) -> HellResult<VulkanSwapchain> {
        let swapchain_support = VulkanSwapchainSupport::new(ctx.phys_device.phys_device, &ctx.surface)?;

        let surface_format = swapchain_support.choose_swap_surface_format()?;
        let swap_present_mode = swapchain_support.choose_swap_present_mode();
        let extent = swapchain_support.choose_swap_extend(window_extent.width, window_extent.height);
        let swap_img_count = swapchain_support.choose_img_count();

        let is_single_queue = ctx.phys_device.queue_support.single_queue()?;
        let queue_indices: Vec<_> = ctx.phys_device.queue_support.indices().into_iter().collect();


        let create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(ctx.surface.surface)
            .min_image_count(swap_img_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)  // always 1, unless for stereoscopic 3D application
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)     // directly render to the images in the swap-chain
            .image_sharing_mode(if is_single_queue { vk::SharingMode::EXCLUSIVE } else { vk::SharingMode::CONCURRENT })
            .queue_family_indices(if is_single_queue { &[] } else { &queue_indices })
            .pre_transform(swapchain_support.capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE) // blend with other windows in the window system?
            .present_mode(swap_present_mode)
            .clipped(true) // ignore color of pixels, that are obscured by other windows
            .old_swapchain(vk::SwapchainKHR::null())
            .build();

        let swapchain_loader = ash::extensions::khr::Swapchain::new(&ctx.instance.instance, &ctx.device.handle);
        let swapchain = unsafe { swapchain_loader.create_swapchain(&create_info, None).expect("failed to create swapchain") };

        let imgs = unsafe { swapchain_loader.get_swapchain_images(swapchain)? };
        let views = VulkanImage::create_img_views(&ctx.device.handle, &imgs, surface_format.format, vk::ImageAspectFlags::COLOR);

        let viewport = [
            vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: extent.width as f32,
                height: extent.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            }
        ];

        let sissor = [
            vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent
            }
        ];

        println!("swapchain created with {} images...", imgs.len());

        Ok(VulkanSwapchain {
            ctx: ctx.clone(),

            vk_swapchain: swapchain,
            swapchain_loader,
            swapchain_support,

            imgs,
            views,

            surface_format,
            extent,

            viewport,
            sissor,

            was_dropped: false,
        })
    }
}

impl VulkanSwapchain {
    pub fn aquire_next_image(&self, img_available_sem: &VulkanSemaphore) -> HellResult<(u32, bool)> {
        unsafe {
            Ok(self.swapchain_loader.acquire_next_image(self.vk_swapchain, u64::MAX, img_available_sem.handle(), vk::Fence::null())?)
        }
    }

    pub fn create_pipeline_viewport_data(&self) -> vk::PipelineViewportStateCreateInfo {
        let viewport_state_info = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&self.viewport)
            .scissors(&self.sissor)
            .build();

        viewport_state_info
    }


    pub fn aspect_ratio(&self) -> f32 {
        self.extent.width as f32 / self.extent.height as f32
    }
}
