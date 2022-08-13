use ash::vk;

use crate::vulkan::buffer::copy_buffer_to_img;
use crate::vulkan::{VulkanCore, VulkanBuffer};

use super::RawImage;

pub struct TextureImage {
    pub img: RawImage,
}

impl TextureImage {
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

        let img = RawImage::new(
            core,
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
        img.transition_image_layout(
            device,
            &core.graphics_cmd_pool,
            &core.device.queues.graphics,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL
        );

        copy_buffer_to_img(core, staging_buffer.buffer, img.img, img_width, img_height);

        // prepare for being read by shader
        img.transition_image_layout(
            device,
            &core.graphics_cmd_pool,
            &core.device.queues.graphics,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
        );

        staging_buffer.drop_manual(device);


        Self { img }
    }
}



impl TextureImage {
    pub fn drop_manual(&self, device: &ash::Device) {
        println!("> dropping VulkanTextureImage");

        self.img.drop_manual(device);
    }
}



// ------------------------------------------------------------------------------------------------
// image
// ------------------------------------------------------------------------------------------------

