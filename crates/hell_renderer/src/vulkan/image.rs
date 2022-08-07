use ash::vk;
use std::{cmp, ptr};

use super::command_buffer::VulkanCommandPool;
use super::vulkan_core::VulkanCore;
use super::buffer;

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
            &core.instance,
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

    pub fn default_for_color_resource(core: &VulkanCore) -> Self {
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

impl VulkanImage {
    pub fn drop_manual(&self, device: &ash::Device) {
        println!("> dropping VulkanImage...");

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

#[allow(clippy::too_many_arguments)]
pub fn transition_img_layout(
    device: &ash::Device,
    cmd_pool: &VulkanCommandPool,
    queue: vk::Queue,
    img: vk::Image,
    mip_levels: u32,
    _format: vk::Format,
    old_layout: vk::ImageLayout,
    new_layout: vk::ImageLayout,
    aspect_mask: vk::ImageAspectFlags,
) {
    let cmd_buffer = cmd_pool.begin_single_time_commands(device);

    let mut barrier = vk::ImageMemoryBarrier {
        s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
        p_next: ptr::null(),
        src_access_mask: vk::AccessFlags::empty(),
        dst_access_mask: vk::AccessFlags::empty(),
        old_layout,
        new_layout,
        src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
        image: img,
        subresource_range: vk::ImageSubresourceRange {
            aspect_mask,
            base_mip_level: 0,
            level_count: mip_levels,
            base_array_layer: 0,
            layer_count: 1,
        },
    };

    let (source_stage, dst_stage) = match (old_layout, new_layout) {
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
            source_stage,
            dst_stage,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier],
        );
    }

    cmd_pool.end_single_time_commands(device, cmd_buffer, queue);
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

pub fn create_img_views(device: &ash::Device, imgs: &[vk::Image], mip_levels: u32, format: vk::Format, aspect_mask: vk::ImageAspectFlags,) -> Vec<vk::ImageView> {
    imgs.iter()
        .map(|&i| create_img_view(device, i, mip_levels, format, aspect_mask))
        .collect()
}

pub fn copy_buffer_to_img(
    device: &ash::Device,
    cmd_pool: &VulkanCommandPool,
    queue: vk::Queue,
    buffer: vk::Buffer,
    img: vk::Image,
    width: u32,
    height: u32,
) {
    let cmd_buffer = cmd_pool.begin_single_time_commands(device);

    let region = vk::BufferImageCopy {
        buffer_offset: 0,

        // tightyly packed pixels
        buffer_row_length: 0,
        buffer_image_height: 0,

        image_subresource: vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            mip_level: 0,
            base_array_layer: 0,
            layer_count: 1,
        },
        image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
        image_extent: vk::Extent3D {
            width,
            height,
            depth: 1,
        },
    };

    unsafe {
        device.cmd_copy_buffer_to_image(
            cmd_buffer,
            buffer,
            img,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &[region],
        );
    }

    cmd_pool.end_single_time_commands(device, cmd_buffer, queue);
}

pub fn create_texture_sampler(
    instance: &ash::Instance,
    device: &ash::Device,
    phys_device: vk::PhysicalDevice,
    mip_levels: u32,
) -> vk::Sampler {
    // TODO: improve
    let props = unsafe { instance.get_physical_device_properties(phys_device) };

    let sampler_info = vk::SamplerCreateInfo {
        s_type: vk::StructureType::SAMPLER_CREATE_INFO,
        p_next: ptr::null(),
        flags: vk::SamplerCreateFlags::empty(),
        mag_filter: vk::Filter::LINEAR,
        min_filter: vk::Filter::LINEAR,
        mipmap_mode: vk::SamplerMipmapMode::LINEAR,
        address_mode_u: vk::SamplerAddressMode::REPEAT,
        address_mode_v: vk::SamplerAddressMode::REPEAT,
        address_mode_w: vk::SamplerAddressMode::REPEAT,
        anisotropy_enable: vk::TRUE,
        max_anisotropy: props.limits.max_sampler_anisotropy,
        compare_enable: vk::FALSE,
        compare_op: vk::CompareOp::ALWAYS,
        mip_lod_bias: 0.0,
        min_lod: 0.0,
        max_lod: mip_levels as f32,
        border_color: vk::BorderColor::INT_OPAQUE_BLACK,
        unnormalized_coordinates: vk::FALSE,
    };

    unsafe {
        device
            .create_sampler(&sampler_info, None)
            .expect("failed to create sampler")
    }
}

#[allow(clippy::too_many_arguments)]
pub fn generate_mipmaps(
    instance: &ash::Instance,
    device: &ash::Device,
    phys_device: vk::PhysicalDevice,
    queue: vk::Queue, // requires graphic capabilityies
    cmd_pool: &VulkanCommandPool,
    img: vk::Image,
    format: vk::Format,
    tex_width: u32,
    tex_height: u32,
    mip_levels: u32,
) {
    let format_props =
        unsafe { instance.get_physical_device_format_properties(phys_device, format) };

    if !format_props
        .optimal_tiling_features
        .contains(vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR)
    {
        panic!("texture image format does not support linear blitting!");
    }

    let cmd_buffer = cmd_pool.begin_single_time_commands(device);

    let mut barrier = vk::ImageMemoryBarrier::builder()
        .image(img)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .subresource_range(
            vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_array_layer(0)
                .layer_count(1)
                .level_count(1)
                .build(),
        )
        .build();

    let mut mip_width = tex_width;
    let mut mip_height = tex_height;

    for i in 1..mip_levels {
        let src_mip_lvl = i - 1;
        let dst_mip_lvl = i;

        barrier.subresource_range.base_mip_level = src_mip_lvl;
        barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
        barrier.new_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
        barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
        barrier.dst_access_mask = vk::AccessFlags::TRANSFER_READ;

        unsafe {
            device.cmd_pipeline_barrier(
                cmd_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            );
        }

        let blit = vk::ImageBlit {
            src_subresource: vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: src_mip_lvl,
                base_array_layer: 0,
                layer_count: 1,
            },
            src_offsets: [
                vk::Offset3D { x: 0, y: 0, z: 0 },
                vk::Offset3D {
                    x: mip_width as i32,
                    y: mip_height as i32,
                    z: 1,
                },
            ],
            dst_subresource: vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: dst_mip_lvl,
                base_array_layer: 0,
                layer_count: 1,
            },
            dst_offsets: [
                vk::Offset3D { x: 0, y: 0, z: 0 },
                vk::Offset3D {
                    x: if mip_width > 1 { mip_width / 2 } else { 1 } as i32,
                    y: if mip_height > 1 { mip_height / 2 } else { 1 } as i32,
                    z: 1,
                },
            ],
        };

        unsafe {
            device.cmd_blit_image(
                cmd_buffer,
                // we are blitting between different levels of the same image -> src & dst image are the same
                img,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                img,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL, // set in create_texture_img()
                &[blit],
                vk::Filter::LINEAR,
            );
        }

        // transition to final layout
        barrier.old_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
        barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
        barrier.src_access_mask = vk::AccessFlags::TRANSFER_READ;
        barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

        unsafe {
            device.cmd_pipeline_barrier(
                cmd_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            );
        }

        // check required for not-square images
        if mip_width > 1 {
            mip_width /= 2;
        }
        if mip_height > 1 {
            mip_height /= 2;
        }
    }

    // transition the last mip-level: the last level is never blitted from
    barrier.subresource_range.base_mip_level = mip_levels - 1;
    barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
    barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
    barrier.src_access_mask = vk::AccessFlags::TRANSFER_READ;
    barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

    unsafe {
        device.cmd_pipeline_barrier(
            cmd_buffer,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[barrier],
        );
    }

    cmd_pool.end_single_time_commands(device, cmd_buffer, queue);
}
