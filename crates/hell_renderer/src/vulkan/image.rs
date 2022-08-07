use ash::vk;
use std::{cmp, ptr};

use super::vulkan_core::Core;
use super::buffer;




// ------------------------------------------------------------------------------------------------
// image
// ------------------------------------------------------------------------------------------------

#[allow(dead_code)]
pub struct Image {
    pub img: vk::Image,
    pub mem: vk::DeviceMemory,
    pub view: vk::ImageView,
    pub mip_levels: u32,
}

impl Image {
    #[allow(dead_code)]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        core: &Core,
        width: u32,
        height: u32,
        mip_level_override: Option<u32>,
        num_samples: vk::SampleCountFlags,
        format: vk::Format,
        tiling: vk::ImageTiling,
        usage: vk::ImageUsageFlags,
        properties: vk::MemoryPropertyFlags,
        aspect_mask: vk::ImageAspectFlags,
    ) -> Self {
        let device = &core.device.vk_device;

        let mip_levels = match mip_level_override {
            Some(m) => m,
            None => calc_mip_levels(width, height),
        };

        let img_info = vk::ImageCreateInfo {
            s_type: vk::StructureType::IMAGE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::ImageCreateFlags::empty(),
            image_type: vk::ImageType::TYPE_2D,
            format,
            extent: vk::Extent3D {
                width,
                height,
                depth: 1,
            },
            mip_levels,
            array_layers: 1,
            samples: num_samples,
            tiling,
            usage,
            sharing_mode: vk::SharingMode::EXCLUSIVE,
            queue_family_index_count: 0,
            p_queue_family_indices: ptr::null(),
            initial_layout: vk::ImageLayout::UNDEFINED,
        };

        let img = unsafe {
            device
                .create_image(&img_info, None)
                .expect("failed to create tex-img")
        };
        let mem_requirements = unsafe { device.get_image_memory_requirements(img) };

        let memory_type_index = buffer::find_memory_type(
            &core.instance.instance,
            core.phys_device.phys_device,
            mem_requirements.memory_type_bits,
            properties,
        );

        let alloc_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            p_next: ptr::null(),
            allocation_size: mem_requirements.size,
            memory_type_index,
        };

        let mem = unsafe {
            device
                .allocate_memory(&alloc_info, None)
                .expect("failed to allocate image memory")
        };

        unsafe {
            device
                .bind_image_memory(img, mem, 0)
                .expect("failed to bind texture img-mem");
        }

        let view = create_img_view(device, img, mip_levels, format, aspect_mask);

        Image {
            img,
            mem,
            view,
            mip_levels,
        }
    }

    #[allow(dead_code)]
    pub fn default_for_color_resource(core: &Core) -> Self {
        Self::new(
            core,
            core.swapchain.extent.width,
            core.swapchain.extent.height,
            Some(1),
            vk::SampleCountFlags::TYPE_1,
            core.swapchain.surface_format.format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSIENT_ATTACHMENT | vk::ImageUsageFlags::COLOR_ATTACHMENT,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            vk::ImageAspectFlags::COLOR
        )
    }
}

impl Image {
    #[allow(dead_code)]
    pub fn drop_manual(&self, device: &ash::Device) {
        println!("> dropping Image...");

        unsafe {
            device.destroy_image_view(self.view, None);
            device.destroy_image(self.img, None);
            device.free_memory(self.mem, None);
        }
    }
}

pub fn calc_mip_levels(width: u32, height: u32) -> u32 {
    (cmp::max(width, height) as f32).log2().floor() as u32 + 1
}




pub fn create_img_view(
    device: &ash::Device,
    img: vk::Image,
    mip_levels: u32,
    format: vk::Format,
    aspect_mask: vk::ImageAspectFlags,
) -> vk::ImageView {
    let view_info = vk::ImageViewCreateInfo {
        s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::ImageViewCreateFlags::empty(),
        image: img,
        view_type: vk::ImageViewType::TYPE_2D,
        format,
        components: vk::ComponentMapping {
            r: vk::ComponentSwizzle::IDENTITY,
            g: vk::ComponentSwizzle::IDENTITY,
            b: vk::ComponentSwizzle::IDENTITY,
            a: vk::ComponentSwizzle::IDENTITY,
        },
        subresource_range: vk::ImageSubresourceRange {
            aspect_mask,
            base_mip_level: 0,
            level_count: mip_levels,
            base_array_layer: 0,
            layer_count: 1,
        },
    };

    unsafe {
        device
            .create_image_view(&view_info, None)
            .expect("failed to create texture-img-view")
    }
}





// ------------------------------------------------------------------------------------------------
// utils
// ------------------------------------------------------------------------------------------------

pub fn create_img_views(device: &ash::Device, imgs: &[vk::Image], mip_levels: u32, format: vk::Format, aspect_mask: vk::ImageAspectFlags,) -> Vec<vk::ImageView> {
    imgs.iter()
        .map(|&i| create_img_view(device, i, mip_levels, format, aspect_mask))
        .collect()
}
