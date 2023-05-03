use ash::vk;
use hell_core::error::HellResult;
use std::ptr;


use crate::vulkan::VulkanContextRef;

use super::{VulkanBuffer, VulkanSwapchain, VulkanCommandPool, VulkanCommands, has_stencil_component, VulkanQueue, VulkanDeviceMemory};


// ----------------------------------------------------------------------------
// vulkan image
// ----------------------------------------------------------------------------

pub struct VulkanImage {
    ctx: VulkanContextRef,
    pub img: vk::Image,
    pub view: vk::ImageView,
    pub mem: VulkanDeviceMemory,
}

impl Drop for VulkanImage {
    fn drop(&mut self) {
        println!("> dropping Image...");

        unsafe {
            let device = &self.ctx.device.handle;
            device.destroy_image_view(self.view, None);
            device.destroy_image(self.img, None);
        }
    }
}

impl VulkanImage {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        ctx: &VulkanContextRef,
        width: usize,
        height: usize,
        num_samples: vk::SampleCountFlags,
        format: vk::Format,
        tiling: vk::ImageTiling,
        usage: vk::ImageUsageFlags,
        properties: vk::MemoryPropertyFlags,
        aspect_mask: vk::ImageAspectFlags,
    ) -> HellResult<Self> {
        let device = &ctx.device.handle;

        let img_info = vk::ImageCreateInfo {
            s_type: vk::StructureType::IMAGE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::ImageCreateFlags::empty(),
            image_type: vk::ImageType::TYPE_2D,
            format,
            extent: vk::Extent3D {
                width: width as u32,
                height: height as u32,
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

        let img = unsafe { device
            .create_image(&img_info, None)
            .expect("failed to create tex-img")
        };

        let mem_requirements = unsafe { device.get_image_memory_requirements(img) };

        let mem = VulkanDeviceMemory::new(ctx, mem_requirements, properties)?;
        mem.bind_to_image(img, 0)?;

        let view = VulkanImage::create_img_view(device, img, format, aspect_mask);

        Ok(VulkanImage {
            ctx: ctx.clone(),
            img,
            mem,
            view,
        })
    }
}



impl VulkanImage {
    pub fn transition_image_layout(&self, device: &ash::Device, cmd_pool: &VulkanCommandPool, queue: &VulkanQueue, format: vk::Format, old_layout: vk::ImageLayout, new_layout: vk::ImageLayout) -> HellResult<()> {
        let cmd_buffer = cmd_pool.begin_single_time_commands(device);

        let subresource_range = vk::ImageSubresourceRange::builder()
            .aspect_mask(VulkanImage::determine_aspect_mask(format, new_layout))
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


    pub fn create_img_views(device: &ash::Device, imgs: &[vk::Image], format: vk::Format, aspect_mask: vk::ImageAspectFlags,) -> Vec<vk::ImageView> {
        imgs.iter()
            .map(|&i| VulkanImage::create_img_view(device, i, format, aspect_mask))
            .collect()
    }
}



// ----------------------------------------------------------------------------
// Texture-Image
// ----------------------------------------------------------------------------

impl VulkanImage {
    pub fn new_tex_img(ctx: &VulkanContextRef, cmds: &VulkanCommands, data: &[u8], img_width: usize, img_height: usize) -> HellResult<Self> {
        let device = &ctx.device.handle;

        let data_size = data.len();
        debug_assert_ne!(data.len(), 0);

        let mut staging_buffer = VulkanBuffer::from_texture_staging(ctx, data_size)?;
        let mem_map = staging_buffer.mem.map_memory(0, data_size, vk::MemoryMapFlags::empty())?;
        mem_map.copy_from_nonoverlapping(data, 0);
        staging_buffer.mem.unmap_memory()?;

        let img = VulkanImage::new(
            ctx,
            img_width,
            img_height,
            vk::SampleCountFlags::TYPE_1,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            vk::ImageAspectFlags::COLOR
        )?;

        // prepare for being copied into
        img.transition_image_layout(
            device,
            &cmds.graphics_pool,
            &ctx.device.queues.graphics,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL
        )?;

        VulkanBuffer::copy_buffer_to_img(ctx, cmds, staging_buffer.handle, img.img, img_width, img_height)?;

        // prepare for being read by shader
        img.transition_image_layout(
            device,
            &cmds.graphics_pool,
            &ctx.device.queues.graphics,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
        )?;

        Ok(img)
    }

    // TODO: do only once
    pub fn new_tex_img_default(ctx: &VulkanContextRef, cmds: &VulkanCommands) -> HellResult<Self> {
        const WIDTH:  usize = 512;
        const HEIGHT: usize = 512;
        const SIZE: usize = WIDTH * HEIGHT;
        const BYTE_SIZE: usize = SIZE * 4;

        println!("creating default tex with size '{}'", BYTE_SIZE);

        const MOD:  u32 = 64;
        const STEP: u32 = MOD / 2;
        let img = image::ImageBuffer::from_fn(WIDTH as u32, HEIGHT as u32, |x, y| {
            if (x % MOD < STEP) && (y & MOD < STEP) {
                image::Rgba([255, 0,   0, 255])
            } else {
                image::Rgba([0,   0, 255, 255])
            }
        });

        Self::new_tex_img(ctx, cmds, img.as_raw().as_slice(), WIDTH, HEIGHT)
    }
}



// ----------------------------------------------------------------------------
// depth image
// ----------------------------------------------------------------------------

impl VulkanImage {
    pub fn new_depth_img(ctx: &VulkanContextRef, swapchain: &VulkanSwapchain, cmds: &VulkanCommands) -> HellResult<Self> {
        let depth_format = ctx.phys_device.depth_format;
        let extent = swapchain.extent;

        let img = VulkanImage::new(
            ctx,
            extent.width as usize,
            extent.height as usize,
            vk::SampleCountFlags::TYPE_1,
            depth_format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            vk::ImageAspectFlags::DEPTH
        )?;

        // Not required: Layout will be transitioned in the renderpass
        img.transition_image_layout(
            &ctx.device.handle,
            &cmds.graphics_pool,
            &ctx.device.queues.graphics,
            depth_format,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        )?;

        Ok(img)
    }
}
