use ash::vk;
use std::ptr;

use hell_common::prelude::*;
use crate::vulkan::phys_device::has_stencil_component;
use crate::vulkan::{VulkanCore, buffer, VulkanQueue, VulkanCommandPool};






pub struct RawImage {
    pub img: vk::Image,
    pub view: vk::ImageView,
    pub mem: vk::DeviceMemory,
}



impl RawImage {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        core: &VulkanCore,
        width: u32,
        height: u32,
        num_samples: vk::SampleCountFlags,
        format: vk::Format,
        tiling: vk::ImageTiling,
        usage: vk::ImageUsageFlags,
        properties: vk::MemoryPropertyFlags,
        aspect_mask: vk::ImageAspectFlags,
    ) -> Self {
        let device = &core.device.device;

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
            mip_levels: 1,
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

        let view = create_img_view(device, img, format, aspect_mask);

        RawImage {
            img,
            mem,
            view,
        }
    }
}



impl RawImage {
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

impl RawImage {
    pub fn transition_image_layout(&self, device: &ash::Device, cmd_pool: &VulkanCommandPool, queue: &VulkanQueue, format: vk::Format, old_layout: vk::ImageLayout, new_layout: vk::ImageLayout) -> HellResult<()> {
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
            .image(self.img)
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

        cmd_pool.end_single_time_commands(device, cmd_buffer, queue.queue)
    }
}





pub fn create_img_view(
    device: &ash::Device,
    img: vk::Image,
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
            level_count: 1,
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

pub fn create_img_views(device: &ash::Device, imgs: &[vk::Image], format: vk::Format, aspect_mask: vk::ImageAspectFlags,) -> Vec<vk::ImageView> {
    imgs.iter()
        .map(|&i| create_img_view(device, i, format, aspect_mask))
        .collect()
}
