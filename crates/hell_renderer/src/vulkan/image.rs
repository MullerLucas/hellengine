use ash::vk;
use std::{cmp, ptr};

use super::buffer::copy_buffer_to_img;
use super::phys_device::has_stencil_component;
use super::vulkan_core::VulkanCore;
use super::{buffer, VulkanBuffer, VulkanCommandPool, VulkanQueue};




// ------------------------------------------------------------------------------------------------
// image
// ------------------------------------------------------------------------------------------------

#[allow(dead_code)]
pub struct VulkanImage {
    pub img: vk::Image,
    pub mem: vk::DeviceMemory,
    pub view: vk::ImageView,
    pub mip_levels: u32,
}

impl VulkanImage {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        core: &VulkanCore,
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
        let device = &core.device.device;

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

        VulkanImage {
            img,
            mem,
            view,
            mip_levels,
        }
    }

    pub fn _default_for_color_resource(core: &VulkanCore) -> Self {
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

    pub fn default_for_texture(core: &VulkanCore, width: u32, height: u32) -> Self {
        Self::new(
            core,
            width,
            height,
            Some(1),
            vk::SampleCountFlags::TYPE_1,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            vk::ImageAspectFlags::COLOR
        )
    }
}

impl VulkanImage {
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

fn transition_image_layout(device: &ash::Device, cmd_pool: &VulkanCommandPool, queue: &VulkanQueue, img: vk::Image, format: vk::Format, old_layout: vk::ImageLayout, new_layout: vk::ImageLayout) {
    let cmd_buffer = cmd_pool.begin_single_time_commands(device);

    let subresource_range = vk::ImageSubresourceRange::builder()
        .aspect_mask(determine_aspect_mask(format, new_layout))
        .base_mip_level(0)
        .level_count(1)
        .base_array_layer(0)
        .layer_count(1)
        .build();

    let mut barrier = vk::ImageMemoryBarrier::builder()
        .old_layout(old_layout)
        .new_layout(new_layout)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .image(img)
        .subresource_range(subresource_range)
        .build();


     let (src_stage, dst_stage) = match (old_layout, new_layout) {
        (vk::ImageLayout::UNDEFINED, vk::ImageLayout::TRANSFER_DST_OPTIMAL) => {
            barrier.src_access_mask = vk::AccessFlags::empty();
            barrier.dst_access_mask = vk::AccessFlags::TRANSFER_WRITE;

            // transfer-stage ^= pseudo-stage, where transfers happen
            (
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
            )
        }
        (vk::ImageLayout::TRANSFER_DST_OPTIMAL, vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL) => {
            barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

            (
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
            )
        }
        (vk::ImageLayout::UNDEFINED, vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL) => {
            barrier.src_access_mask = vk::AccessFlags::empty();
            barrier.dst_access_mask = vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ
                | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE;

            // reading: EARLY_FRAGMENT_TEST stage - writing: LATE_FRAGMENT_TEST stage => pick earliest stage
            (
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            )
        }
        _ => {
            panic!("unsuported layout transition!");
        }
    };



    unsafe {
        device.cmd_pipeline_barrier(
            cmd_buffer,
            src_stage,
            dst_stage,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier],
        );
    }

    cmd_pool.end_single_time_commands(device, cmd_buffer, queue.queue);
}

fn determine_aspect_mask(format: vk::Format, layout: vk::ImageLayout) -> vk::ImageAspectFlags {
    if layout == vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL {
        if has_stencil_component(format) {
            vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL
        } else {
            vk::ImageAspectFlags::DEPTH
        }
    } else {
        vk::ImageAspectFlags::COLOR
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




// ------------------------------------------------------------------------------------------------
// texture-image
// ------------------------------------------------------------------------------------------------

pub struct VulkanTextureImage {
    pub img: VulkanImage,
}

impl VulkanTextureImage {
    // TODO: error handling
    pub fn new(core: &VulkanCore, path: impl AsRef<std::path::Path>) -> Self {
        let device = &core.device.device;

        let raw_img = image::open(path).unwrap();
        raw_img.flipv();

        let img_width = raw_img.width();
        let img_height = raw_img.height();
        let img_size = (std::mem::size_of::<u8>() as u32 * img_width * img_height * 4) as vk::DeviceSize;

        if img_size == 0 {
            panic!("failed to load image at");
        }

        let img_data = match &raw_img {
            image::DynamicImage::ImageLuma8(_) | image::DynamicImage::ImageRgb8(_) => {
                raw_img.to_rgba8().into_raw()
            },
            image::DynamicImage::ImageLumaA8(_) | image::DynamicImage::ImageRgba8(_) => {
                raw_img.into_bytes()
            }
            _ => { panic!("invalid image format"); }
        };

        let staging_buffer = VulkanBuffer::from_texture_staging(core, img_size);

        unsafe {
            let data_ptr = device.map_memory(staging_buffer.mem, 0, img_size, vk::MemoryMapFlags::empty()).unwrap() as *mut u8;
            data_ptr.copy_from_nonoverlapping(img_data.as_ptr(), img_data.len());
            device.unmap_memory(staging_buffer.mem);
        }

        let img = VulkanImage::default_for_texture(core, img_width, img_height);

        // prepare for being copied into
        transition_image_layout(
            device,
            &core.graphics_cmd_pool,
            &core.device.queues.graphics,
            img.img,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL
        );

        copy_buffer_to_img(core, staging_buffer.buffer, img.img, img_width, img_height);

        // prepare for being read by shader
        transition_image_layout(
            device,
            &core.graphics_cmd_pool,
            &core.device.queues.graphics,
            img.img,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
        );

        staging_buffer.drop_manual(device);


        Self {
            img
        }
    }
}

impl VulkanTextureImage {
    pub fn drop_manual(&self, device: &ash::Device) {
        println!("> dropping VulkanTextureImage");

        self.img.drop_manual(device);
    }
}



// ------------------------------------------------------------------------------------------------
// image
// ------------------------------------------------------------------------------------------------

pub struct DepthImage {
    pub img: VulkanImage,
}

impl DepthImage {
    pub fn new(core: &VulkanCore) -> Self {
        let depth_format = core.phys_device.depth_format;

        let extent = core.swapchain.extent;

        let img = VulkanImage::new(
            core,
            extent.width,
            extent.height,
            Some(1),
            vk::SampleCountFlags::TYPE_1,
            depth_format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            vk::ImageAspectFlags::DEPTH
        );

        // Not required: Layout will be transitioned in the renderpass
        transition_image_layout(
            &core.device.device,
            &core.graphics_cmd_pool,
            &core.device.queues.graphics,
            img.img,
            depth_format,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        );


        Self {
            img
        }
    }
}

impl DepthImage {
    pub fn drop_manual(&self, device: &ash::Device) {
        println!("> dropping DepthImage...");

        self.img.drop_manual(device);
    }
}
