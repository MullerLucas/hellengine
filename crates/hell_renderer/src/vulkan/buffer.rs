use std::{ptr, mem};
use ash::vk;

use super::command_buffer::VulkanCommandPool;
use super::{config, VulkanSampler};
use super::descriptors::VulkanDescriptorPool;
use super::image::VulkanTextureImage;
use super::vertext::Vertex;
use super::vulkan_core::VulkanCore;



// ----------------------------------------------------------------------------
// buffer
// ----------------------------------------------------------------------------

pub struct VulkanBuffer {
    pub buffer: vk::Buffer,
    pub mem: vk::DeviceMemory,
    pub size: vk::DeviceSize,
}

impl VulkanBuffer {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        core: &VulkanCore, size: vk::DeviceSize, usage: vk::BufferUsageFlags, properties: vk::MemoryPropertyFlags, sharing_mode: vk::SharingMode, queue_family_indices: Option<&[u32]>
    ) -> Self {
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
            &core.instance.instance,
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

        let buffer_size = std::mem::size_of_val(vertices) as vk::DeviceSize;
        println!("VERT-SIZE: {}", buffer_size);

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
            Some(&[core.device.queues.graphics.family_idx, core.device.queues.transfer.family_idx])
        );

        copy_buffer(
            device, &core.transfer_cmd_pool, core.device.queues.transfer.queue,
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
            Some(&[core.device.queues.graphics.family_idx, core.device.queues.transfer.family_idx])
        );

        copy_buffer(device, &core.transfer_cmd_pool, core.device.queues.transfer.queue, &staging_buffer, &device_buffer);

        unsafe {
            // TODO: implement Drop for VulkanBuffer
            device.destroy_buffer(staging_buffer.buffer, None);
            device.free_memory(staging_buffer.mem, None);
        }



        device_buffer
    }

    pub fn from_uniform(core: &VulkanCore) -> Self {
        VulkanBuffer::new(
            core,
            VulkanUniformBufferObject::device_size(),
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            vk::SharingMode::EXCLUSIVE,
            None,
        )
    }

    pub fn from_texture_staging(core: &VulkanCore, img_size: u64) -> Self {
        VulkanBuffer::new(
            core,
            img_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            vk::SharingMode::EXCLUSIVE,
            None
        )
    }
}


impl VulkanBuffer {
    pub fn drop_manual(&self, device: &ash::Device) {
        println!("> dropping Buffer...");

        unsafe {
            device.destroy_buffer(self.buffer, None);
            device.free_memory(self.mem, None);
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

pub fn copy_buffer_to_img(core: &VulkanCore, buffer: vk::Buffer, img: vk::Image, width: u32, height: u32) {
    let device = &core.device.device;
    let cmd_buffer = core.transfer_cmd_pool.begin_single_time_commands(device);

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

    core.transfer_cmd_pool.end_single_time_commands(device, cmd_buffer, core.device.queues.transfer.queue);
}




// ----------------------------------------------------------------------------
// uniform-buffer
// ----------------------------------------------------------------------------


#[allow(dead_code)]
pub struct VulkanUniformBufferObject {
    model: glam::Mat4,
    view: glam::Mat4,
    proj: glam::Mat4,
}

impl VulkanUniformBufferObject {
    pub fn device_size() -> vk::DeviceSize {
        std::mem::size_of::<Self>() as vk::DeviceSize
    }
}




pub struct VulkanUniformData {
    pub ubo: VulkanUniformBufferObject,
    pub uniform_buffers_per_frame: Vec<VulkanBuffer>,

    pub texture: VulkanTextureImage,
    pub sampler: VulkanSampler,

    pub descriptor_pool: VulkanDescriptorPool
}

impl VulkanUniformData {
    pub fn new(core: &VulkanCore, aspect_ratio: f32) -> Self {
        let device = &core.device.device;

        let ubo = VulkanUniformBufferObject {
            model: glam::Mat4::IDENTITY,
            view: glam::Mat4::look_at_rh(glam::vec3(2.0, 2.0, 2.0), glam::vec3(0.0, 0.0, 0.0), glam::vec3(0.0, 0.0, 1.0)),
            proj: {
                // opengl -> y coord of the clip coords is inverted -> flip sign of scaling factor of the y-axis in the proj-matrix
                let mut proj = glam::Mat4::perspective_rh(f32::to_radians(90.0), aspect_ratio, 0.1, 10.0);
                proj.y_axis.y *= -1.0;
                proj
            }
        };

        let uniform_buffers_per_frame: Vec<_> = (0..config::MAX_FRAMES_IN_FLIGHT)
            .into_iter()
            .map(|_| VulkanBuffer::from_uniform(core))
            .collect();

        let texture = VulkanTextureImage::new(core, config::TEXTURE_PATH);
        let sampler = VulkanSampler::new(core).unwrap();

        let descriptor_pool = VulkanDescriptorPool::new(device, &uniform_buffers_per_frame, &texture, &sampler).unwrap();

        Self {
            ubo,
            uniform_buffers_per_frame,

            texture,
            sampler,

            descriptor_pool
        }
    }
}

impl VulkanUniformData {
    pub fn drop_manual(&self, device: &ash::Device) {
        println!("> dropping UniformData...");

        self.uniform_buffers_per_frame.iter().for_each(|b| b.drop_manual(device));

        self.texture.drop_manual(device);
        self.sampler.drop_manual(device);

        self.descriptor_pool.drop_manual(device);
    }
}


impl VulkanUniformData {
    // TODO: error handling
    pub fn update_uniform_buffer(&mut self, core: &VulkanCore, img_idx: usize, delta_time: f32) {
        let device = &core.device.device;

        let angle = f32::to_radians(90.0) * (delta_time / 20.0);
        self.ubo.model = glam::Mat4::from_rotation_z(angle);
        // self.ubo.model = glam::Mat4::IDENTITY;

        let buff_size = std::mem::size_of::<VulkanUniformBufferObject>() as u64;
        let uniform_buffer = &self.uniform_buffers_per_frame[img_idx];


        unsafe {
            let data_ptr = device.map_memory(uniform_buffer.mem, 0, buff_size, vk::MemoryMapFlags::empty()).unwrap() as *mut VulkanUniformBufferObject;
            data_ptr.copy_from_nonoverlapping(&self.ubo, 1);
            device.unmap_memory(uniform_buffer.mem);
        }
    }
}
