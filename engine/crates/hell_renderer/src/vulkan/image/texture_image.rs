use ash::vk;
use hell_error::{HellResult, ErrToHellErr};
use hell_resources::resources::TextureResource;

use crate::vulkan::{VulkanCtxRef, buffer::{VulkanBuffer, copy_buffer_to_img}, command_buffer::VulkanCommands};

use super::RawImage;



#[derive(Clone)]
pub struct TextureImage {
    pub img: RawImage,
}

impl TextureImage {
    pub fn from(ctx: &VulkanCtxRef, cmds: &VulkanCommands, img_res: &TextureResource) -> HellResult<Self> {
        let device = &ctx.device.device;

        let img = img_res.get_img();
        let img_data = img.as_raw();
        let img_width = img.width();
        let img_height = img.height();
        let img_size = (std::mem::size_of::<u8>() as u32 * img_width * img_height * 4) as vk::DeviceSize;

        if img_size == 0 {
            panic!("failed to load image at");
        }

        let staging_buffer = VulkanBuffer::from_texture_staging(ctx, img_size);

        unsafe {
            let data_ptr = device.map_memory(staging_buffer.mem, 0, img_size, vk::MemoryMapFlags::empty()).to_render_hell_err()? as *mut u8;
            data_ptr.copy_from_nonoverlapping(img_data.as_ptr(), img_data.len());
            device.unmap_memory(staging_buffer.mem);
        }

        let raw_img = RawImage::new(
            ctx,
            img_width,
            img_height,
            vk::SampleCountFlags::TYPE_1,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            vk::ImageAspectFlags::COLOR
        );

        // prepare for being copied into
        raw_img.transition_image_layout(
            device,
            &cmds.graphics_pool,
            &ctx.device.queues.graphics,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL
        )?;

        copy_buffer_to_img(ctx, cmds, staging_buffer.buffer, raw_img.img, img_width, img_height)?;

        // prepare for being read by shader
        raw_img.transition_image_layout(
            device,
            &cmds.graphics_pool,
            &ctx.device.queues.graphics,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
        )?;

        Ok(Self { img: raw_img })
    }
}
