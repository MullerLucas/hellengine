use std::{ptr, mem};
use ash::vk;

use super::command_buffer::VulkanCommandPool;
use super::vertext::Vertex;
use super::vulkan_core::VulkanCore;


pub struct VulkanBuffer {
    pub buffer: vk::Buffer,
    pub mem: vk::DeviceMemory,
    pub size: vk::DeviceSize,
}

impl VulkanBuffer {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        core: &VulkanCore, size: vk::DeviceSize,
        usage: vk::BufferUsageFlags, properties: vk::MemoryPropertyFlags, sharing_mode: vk::SharingMode, queue_family_indices: Option<&[u32]>)
    -> Self {
        let device = &core.device.device;

        let mut buffer_info = vk::BufferCreateInfo {
            s_type: vk::StructureType::BUFFER_CREATE_INFO,
            p_next: ptr::null(),
            flags: Default::default(),
            size,
            usage,
            sharing_mode,
            queue_family_index_count: 0,
            p_queue_family_indices: ptr::null(),
        };

        if let Some(indices) = queue_family_indices {
            buffer_info.queue_family_index_count = indices.len() as u32;
            buffer_info.p_queue_family_indices = indices.as_ptr();
        }

        let buffer = unsafe { device.create_buffer(&buffer_info, None) }
            .expect("failed to create vertex-buffer");

        let mem_requirements = unsafe { device.get_buffer_memory_requirements(buffer) };


        let mem_type_idx = find_memory_type(
            &core.instance,
            core.phys_device.phys_device,
            mem_requirements.memory_type_bits,
            properties,
        );

        let alloc_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            p_next: ptr::null(),
            allocation_size: mem_requirements.size,
            memory_type_index: mem_type_idx
        };

        let mem = unsafe { device.allocate_memory(&alloc_info, None) }
            .expect("failed to allocate vertex memory");

        unsafe { device.bind_buffer_memory(buffer, mem, 0) }
            .expect("failed to bind vertex-buffer");



        Self {
            buffer,
            mem,
            size
        }
    }

    // TODO: error handling
    pub fn from_vertices(core: &VulkanCore, vertices: &[Vertex]) -> Self {
        let device = &core.device.device;

        let buffer_size = Vertex::get_device_size();

        let staging_buffer = VulkanBuffer::new(
            core,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            vk::SharingMode::EXCLUSIVE,
            None
        );

        unsafe {
            let mem_ptr = device.map_memory(staging_buffer.mem, 0, buffer_size, vk::MemoryMapFlags::empty()).unwrap() as *mut Vertex;
            mem_ptr.copy_from_nonoverlapping(vertices.as_ptr(), vertices.len());
            device.unmap_memory(staging_buffer.mem);
        }

        let device_buffer = VulkanBuffer::new(
            core,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            vk::SharingMode::CONCURRENT, // TODO: not optimal
            Some(&[core.device.queues.graphics_idx, core.device.queues.transfer_idx])
        );

        copy_buffer(
            device, &core.transfer_cmd_pool, core.device.queues.transfer_queue,
            &staging_buffer, &device_buffer
        );

        unsafe {
            // TODO: implement drop for VulkanBuffer
            device.destroy_buffer(staging_buffer.buffer, None);
            device.free_memory(staging_buffer.mem, None);
        }



        device_buffer
    }

    pub fn from_indices(core: &VulkanCore, indices: &[u32]) -> Self {
        let device = &core.device.device;

        let buffer_size = mem::size_of_val(indices) as vk::DeviceSize;

        let staging_buffer = VulkanBuffer::new(
            core,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            vk::SharingMode::EXCLUSIVE,
            None
        );

        unsafe {
            let mem_ptr = device.map_memory(staging_buffer.mem, 0, buffer_size, vk::MemoryMapFlags::empty()).unwrap() as *mut u32;
            mem_ptr.copy_from_nonoverlapping(indices.as_ptr(), indices.len());
            device.unmap_memory(staging_buffer.mem);
        }

        let device_buffer = VulkanBuffer::new(
            core,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
            vk::SharingMode::CONCURRENT,
            Some(&[core.device.queues.graphics_idx, core.device.queues.transfer_idx])
        );

        copy_buffer(device, &core.transfer_cmd_pool, core.device.queues.transfer_queue, &staging_buffer, &device_buffer);

        unsafe {
            // TODO: implement Drop for VulkanBuffer
            device.destroy_buffer(staging_buffer.buffer, None);
            device.free_memory(staging_buffer.mem, None);
        }



        device_buffer
    }
}


impl VulkanBuffer {
    pub fn drop_manual(&self, device: &ash::Device) {
        println!("> droping buffer...");

        unsafe {
            device.destroy_buffer(self.buffer, None);
        }
    }
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

fn copy_buffer(device: &ash::Device, cmd_pool: &VulkanCommandPool, queue: vk::Queue, src_buff: &VulkanBuffer, dst_buff: &VulkanBuffer) {
    let command_buffer = cmd_pool.begin_single_time_commands(device);

    let copy_region = vk::BufferCopy {
        src_offset: 0,
        dst_offset: 0,
        size: src_buff.size,
    };

    unsafe {
        device.cmd_copy_buffer(command_buffer, src_buff.buffer, dst_buff.buffer, &[copy_region]);
    }

    cmd_pool.end_single_time_commands(device, command_buffer, queue);
}
