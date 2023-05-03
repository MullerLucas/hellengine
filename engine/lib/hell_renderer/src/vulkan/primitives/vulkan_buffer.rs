use ash::vk;
use hell_core::error::HellResult;
use crate::vulkan::{VulkanContextRef, Vertex3D};

use super::{VulkanCommands, VulkanCommandPool, VulkanDeviceMemory};





// ----------------------------------------------------------------------------
// buffer
// ----------------------------------------------------------------------------

#[derive(Debug)]
pub struct VulkanBuffer {
    ctx: VulkanContextRef,
    pub size: usize,
    pub handle: vk::Buffer,
    pub mem: VulkanDeviceMemory,
}

impl Drop for VulkanBuffer {
    fn drop(&mut self) {
        println!("> dropping VulkanBuffer...");

        unsafe {
            let device = &self.ctx.device.handle;
            device.destroy_buffer(self.handle, None);
        }
    }
}

impl VulkanBuffer {
    #[allow(clippy::too_many_arguments)]
    pub fn new(ctx: &VulkanContextRef, size: usize, usage: vk::BufferUsageFlags, properties: vk::MemoryPropertyFlags, sharing_mode: vk::SharingMode, queue_family_indices: Option<&[u32]>) -> HellResult<Self> {
        let device = &ctx.device.handle;

        let mut buffer_info = vk::BufferCreateInfo {
            s_type: vk::StructureType::BUFFER_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: Default::default(),
            size: size as vk::DeviceSize,
            usage,
            sharing_mode,
            queue_family_index_count: 0,
            p_queue_family_indices: std::ptr::null(),
        };

        if let Some(indices) = queue_family_indices {
            buffer_info.queue_family_index_count = indices.len() as u32;
            buffer_info.p_queue_family_indices = indices.as_ptr();
        }

        let buffer = unsafe { device.create_buffer(&buffer_info, None) }?;
        let mem_requirements = VulkanDeviceMemory::create_buffer_requirements(ctx, buffer);
        let mem = VulkanDeviceMemory::new(ctx, mem_requirements, properties)?;
        mem.bind_to_buffer(buffer)?;

        Ok(Self {
            ctx: ctx.clone(),
            handle: buffer,
            mem,
            size
        })
    }

    pub fn from_vertices(ctx: &VulkanContextRef, cmds: &VulkanCommands, vertices: &[Vertex3D]) -> HellResult<Self> {
        let device = &ctx.device.handle;

        let buffer_size = std::mem::size_of_val(vertices);
        println!("VERT-SIZE: {}", buffer_size);

        let mut staging_buffer = VulkanBuffer::new(
            ctx,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            vk::SharingMode::EXCLUSIVE,
            None
        )?;

        let mem_map = staging_buffer.mem.map_memory(0, buffer_size, vk::MemoryMapFlags::empty())?;
        mem_map.copy_from_nonoverlapping(vertices, 0);
        staging_buffer.mem.unmap_memory()?;

        let device_buffer = VulkanBuffer::new(
            ctx,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            vk::SharingMode::CONCURRENT, // TODO: not optimal
            Some(&[ctx.device.queues.graphics.family_idx, ctx.device.queues.transfer.family_idx])
        )?;

        Self::copy_buffer(
            device,
            &cmds.transfer_pool,
            ctx.device.queues.transfer.queue,
            &staging_buffer,
            &device_buffer
        )?;

        Ok(device_buffer)
    }

    pub fn from_indices(ctx: &VulkanContextRef, cmds: &VulkanCommands, indices: &[u32]) -> HellResult<Self> {
        let device = &ctx.device.handle;

        let buffer_size = std::mem::size_of_val(indices);

        let mut staging_buffer = VulkanBuffer::new(
            ctx,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            vk::SharingMode::EXCLUSIVE,
            None
        )?;

        let mem_map = staging_buffer.mem.map_memory(0, buffer_size, vk::MemoryMapFlags::empty())?;
        mem_map.copy_from_nonoverlapping(indices, 0);
        staging_buffer.mem.unmap_memory()?;

        let device_buffer = VulkanBuffer::new(
            ctx,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            vk::SharingMode::CONCURRENT,
            Some(&[ctx.device.queues.graphics.family_idx, ctx.device.queues.transfer.family_idx])
        )?;

        Self::copy_buffer(device, &cmds.transfer_pool, ctx.device.queues.transfer.queue, &staging_buffer, &device_buffer)?;

        Ok(device_buffer)
    }

    pub fn from_uniform(ctx: &VulkanContextRef, size: usize) -> HellResult<Self> {
        VulkanBuffer::new(
            ctx,
            size,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            vk::SharingMode::EXCLUSIVE,
            None,
        )
    }

    pub fn from_storage(ctx: &VulkanContextRef, size: usize) -> HellResult<Self> {
        VulkanBuffer::new(
            ctx,
            size,
            vk::BufferUsageFlags::STORAGE_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            vk::SharingMode::EXCLUSIVE,
            None,
        )
    }

    pub fn from_texture_staging(ctx: &VulkanContextRef, img_size: usize) -> HellResult<Self> {
        VulkanBuffer::new(
            ctx,
            img_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            // TODO: check support for HOST_COHERENT | DEVICE_LOCAL, and use it
            // vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::DEVICE_LOCAL,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            vk::SharingMode::EXCLUSIVE,
            None
        )
    }
}

impl VulkanBuffer {
    fn copy_buffer(device: &ash::Device, cmd_pool: &VulkanCommandPool, queue: vk::Queue, src_buff: &VulkanBuffer, dst_buff: &VulkanBuffer) -> HellResult<()> {
        let command_buffer = cmd_pool.begin_single_time_commands(device);

        let copy_region = vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size: src_buff.size as vk::DeviceSize,
        };

        unsafe {
            device.cmd_copy_buffer(command_buffer, src_buff.handle, dst_buff.handle, &[copy_region]);
        }

        cmd_pool.end_single_time_commands(device, command_buffer, queue)?;

        Ok(())
    }

    pub fn copy_buffer_to_img(ctx: &VulkanContextRef, cmds: &VulkanCommands, buffer: vk::Buffer, img: vk::Image, width: usize, height: usize) -> HellResult<()> {
        let device = &ctx.device.handle;
        let cmd_buffer = cmds.transfer_pool.begin_single_time_commands(device);

        let img_subresource = vk::ImageSubresourceLayers::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .mip_level(0)
            .base_array_layer(0)
            .layer_count(1)
            .build();

        let img_offset = vk::Offset3D { x: 0, y: 0, z: 0 };
        let img_extent = vk::Extent3D {
            width: width as u32,
            height: height as u32,
            depth: 1
        };

        let region = vk::BufferImageCopy::builder()
            .buffer_offset(0)
            .buffer_row_length(0)
            .buffer_image_height(0)
            .image_subresource(img_subresource)
            .image_offset(img_offset)
            .image_extent(img_extent)
            .build();

        unsafe {
            device.cmd_copy_buffer_to_image(cmd_buffer, buffer, img, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[region]);
        }

        cmds.transfer_pool.end_single_time_commands(device, cmd_buffer, ctx.device.queues.transfer.queue)?;

        Ok(())
    }
}
