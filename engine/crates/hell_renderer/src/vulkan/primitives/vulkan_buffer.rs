use ash::prelude::VkResult;
use ash::vk;
use hell_error::{HellResult, ErrToHellErr};

use crate::vulkan::{VulkanContextRef, Vertex3D, shader::VulkanUboData};

use super::{VulkanCommands, VulkanCommandPool};






// ----------------------------------------------------------------------------
// buffer
// ----------------------------------------------------------------------------

#[derive(Debug)]
pub struct VulkanBuffer {
    ctx: VulkanContextRef,
    pub buffer: vk::Buffer,
    pub mem: vk::DeviceMemory,
    pub size: vk::DeviceSize,
}

impl Drop for VulkanBuffer {
    fn drop(&mut self) {
        println!("> dropping VulkanBuffer...");

        unsafe {
            let device = &self.ctx.device.handle;
            device.destroy_buffer(self.buffer, None);
            device.free_memory(self.mem, None);
        }
    }
}

impl VulkanBuffer {
    #[allow(clippy::too_many_arguments)]
    pub fn new(ctx: &VulkanContextRef, size: vk::DeviceSize, usage: vk::BufferUsageFlags, properties: vk::MemoryPropertyFlags, sharing_mode: vk::SharingMode, queue_family_indices: Option<&[u32]>) -> Self {
        let device = &ctx.device.handle;

        let mut buffer_info = vk::BufferCreateInfo {
            s_type: vk::StructureType::BUFFER_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: Default::default(),
            size,
            usage,
            sharing_mode,
            queue_family_index_count: 0,
            p_queue_family_indices: std::ptr::null(),
        };

        if let Some(indices) = queue_family_indices {
            buffer_info.queue_family_index_count = indices.len() as u32;
            buffer_info.p_queue_family_indices = indices.as_ptr();
        }

        let buffer = unsafe { device.create_buffer(&buffer_info, None) }
            .expect("failed to create vertex-buffer");

        let mem_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

        let mem_type_idx = Self::find_memory_type(
            &ctx.instance.instance,
            ctx.phys_device.phys_device,
            mem_requirements.memory_type_bits,
            properties,
        );

        let alloc_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            p_next: std::ptr::null(),
            allocation_size: mem_requirements.size,
            memory_type_index: mem_type_idx
        };

        let mem = unsafe { device.allocate_memory(&alloc_info, None) }.expect("failed to allocate vertex memory");
        unsafe { device.bind_buffer_memory(buffer, mem, 0) }.expect("failed to bind vertex-buffer");

        Self {
            ctx: ctx.clone(),
            buffer,
            mem,
            size
        }
    }

    pub fn from_vertices(ctx: &VulkanContextRef, cmds: &VulkanCommands, vertices: &[Vertex3D]) -> HellResult<Self> {
        let device = &ctx.device.handle;

        let buffer_size = std::mem::size_of_val(vertices) as vk::DeviceSize;
        println!("VERT-SIZE: {}", buffer_size);

        let staging_buffer = VulkanBuffer::new(
            ctx,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            vk::SharingMode::EXCLUSIVE,
            None
        );

        unsafe {
            let mem_ptr = device.map_memory(staging_buffer.mem, 0, buffer_size, vk::MemoryMapFlags::empty()).to_render_hell_err()? as *mut Vertex3D;
            mem_ptr.copy_from_nonoverlapping(vertices.as_ptr(), vertices.len());
            device.unmap_memory(staging_buffer.mem);
        }

        let device_buffer = VulkanBuffer::new(
            ctx,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            vk::SharingMode::CONCURRENT, // TODO: not optimal
            Some(&[ctx.device.queues.graphics.family_idx, ctx.device.queues.transfer.family_idx])
        );

        Self::copy_buffer(
            device, &cmds.transfer_pool, ctx.device.queues.transfer.queue,
            &staging_buffer, &device_buffer
        )?;

        Ok(device_buffer)
    }

    pub fn from_indices(ctx: &VulkanContextRef, cmds: &VulkanCommands, indices: &[u32]) -> HellResult<Self> {
        let device = &ctx.device.handle;

        let buffer_size = std::mem::size_of_val(indices) as vk::DeviceSize;

        let staging_buffer = VulkanBuffer::new(
            ctx,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            vk::SharingMode::EXCLUSIVE,
            None
        );

        unsafe {
            let mem_ptr = device.map_memory(staging_buffer.mem, 0, buffer_size, vk::MemoryMapFlags::empty()).to_render_hell_err()? as *mut u32;
            mem_ptr.copy_from_nonoverlapping(indices.as_ptr(), indices.len());
            device.unmap_memory(staging_buffer.mem);
        }

        let device_buffer = VulkanBuffer::new(
            ctx,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            vk::SharingMode::CONCURRENT,
            Some(&[ctx.device.queues.graphics.family_idx, ctx.device.queues.transfer.family_idx])
        );

        Self::copy_buffer(device, &cmds.transfer_pool, ctx.device.queues.transfer.queue, &staging_buffer, &device_buffer)?;

        Ok(device_buffer)
    }

    pub fn from_uniform(ctx: &VulkanContextRef, size: vk::DeviceSize) -> Self {
        VulkanBuffer::new(
            ctx,
            size,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            vk::SharingMode::EXCLUSIVE,
            None,
        )
    }

    pub fn from_storage(ctx: &VulkanContextRef, size: vk::DeviceSize) -> Self {
        VulkanBuffer::new(
            ctx,
            size,
            vk::BufferUsageFlags::STORAGE_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            vk::SharingMode::EXCLUSIVE,
            None,
        )
    }

    pub fn from_texture_staging(ctx: &VulkanContextRef, img_size: u64) -> Self {
        VulkanBuffer::new(
            ctx,
            img_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            vk::SharingMode::EXCLUSIVE,
            None
        )
    }
}




impl VulkanBuffer {
    pub fn upload_data_buffer<T: VulkanUboData>(&self, device: &ash::Device, data: &T) -> HellResult<()> {
        let buff_size = T::device_size();

        unsafe {
            let data_ptr = device.map_memory(self.mem, 0, buff_size, vk::MemoryMapFlags::empty()).to_render_hell_err()? as *mut T;
            data_ptr.copy_from_nonoverlapping(data, 1);
            device.unmap_memory(self.mem);
        }

        Ok(())
    }

    pub fn upload_data_buffer_array<T: VulkanUboData>(&self, device: &ash::Device, min_ubo_alignment: u64, data: &T, idx: usize) -> VkResult<()> {
        let offset = T::padded_device_size(min_ubo_alignment) * idx as u64;
        let buff_size = T::device_size();

        unsafe {
            let data_ptr = device.map_memory(self.mem, offset, buff_size, vk::MemoryMapFlags::empty())? as *mut T;
            data_ptr.copy_from_nonoverlapping(data, 1);
            device.unmap_memory(self.mem);
        }

        Ok(())
    }

    /// # Safety
    /// There is no safety don't use this function :)
    pub unsafe fn upload_data_storage_buffer<T: VulkanUboData>(&self, device: &ash::Device, data: *const T, data_count: usize) -> HellResult<()> {
        let buff_size = T::device_size() * data_count as u64;

        let data_ptr = device.map_memory(self.mem, 0, buff_size, vk::MemoryMapFlags::empty()).to_render_hell_err()? as *mut T;
        data_ptr.copy_from_nonoverlapping(data, data_count);
        device.unmap_memory(self.mem);

        Ok(())
    }

}

impl<'a> VulkanBuffer {
    pub fn map_memory(&self, device: &'a ash::Device, offset: u64, buff_size: u64, mem_map_flags: vk::MemoryMapFlags) -> VkResult<DeviceMemoryMapGuard<'a>> {
        DeviceMemoryMapGuard::new(device, self.mem, offset, buff_size, mem_map_flags)
    }

    pub fn find_memory_type(instance: &ash::Instance, phys_device: vk::PhysicalDevice, type_filter: u32, properties: vk::MemoryPropertyFlags) -> u32 {
        let mem_props = unsafe { instance.get_physical_device_memory_properties(phys_device) };

        for (i, mem_type) in mem_props.memory_types.iter().enumerate() {
            if (type_filter & (1 << i) > 0) && mem_type.property_flags.contains(properties)  {
                return i as u32;
            }
        }

        panic!("failed to find suitable memory-type");
    }

    fn copy_buffer(device: &ash::Device, cmd_pool: &VulkanCommandPool, queue: vk::Queue, src_buff: &VulkanBuffer, dst_buff: &VulkanBuffer) -> HellResult<()> {
        let command_buffer = cmd_pool.begin_single_time_commands(device);

        let copy_region = vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size: src_buff.size,
        };

        unsafe {
            device.cmd_copy_buffer(command_buffer, src_buff.buffer, dst_buff.buffer, &[copy_region]);
        }

        cmd_pool.end_single_time_commands(device, command_buffer, queue)?;

        Ok(())
    }

    pub fn copy_buffer_to_img(ctx: &VulkanContextRef, cmds: &VulkanCommands, buffer: vk::Buffer, img: vk::Image, width: u32, height: u32) -> HellResult<()> {
        let device = &ctx.device.handle;
        let cmd_buffer = cmds.transfer_pool.begin_single_time_commands(device);

        let img_subresource = vk::ImageSubresourceLayers::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .mip_level(0)
            .base_array_layer(0)
            .layer_count(1)
            .build();

        let img_offset = vk::Offset3D { x: 0, y: 0, z: 0 };
        let img_extent = vk::Extent3D { width, height, depth: 1 };

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




// ----------------------------------------------------------------------------
// DeviceMemoryGuard
// ----------------------------------------------------------------------------

pub struct DeviceMemoryMapGuard<'a> {
    device: &'a ash::Device,
    mem: vk::DeviceMemory,
    data_ptr: *mut std::os::raw::c_void,
}

impl<'a> DeviceMemoryMapGuard<'a> {
    pub fn new(device: &'a ash::Device, mem: vk::DeviceMemory, offset: vk::DeviceSize, buff_size: vk::DeviceSize, mem_map_flags: vk::MemoryMapFlags) -> VkResult<Self> {
        let data_ptr = unsafe { device.map_memory(mem, offset, buff_size, mem_map_flags)? };

        Ok(Self {
            device,
            mem,
            data_ptr
        })
    }

    pub fn data_as<T>(&mut self) -> &mut T {
        unsafe {
            &mut *(self.data_ptr as *mut T)
        }
    }
}

impl Drop for DeviceMemoryMapGuard<'_> {
    fn drop(&mut self) {
        // data_ptr.copy_from_nonoverlapping(data, 1);
        unsafe {
            self.device.unmap_memory(self.mem);
        }
    }
}


