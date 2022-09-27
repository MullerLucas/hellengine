use std::ptr;

use ash::vk;

use super::config;
use super::logic_device::VulkanLogicDevice;




pub struct VulkanCommandPool {
    pub pool: vk::CommandPool,
    pub cmd_buffers: Vec<vk::CommandBuffer>
}

impl VulkanCommandPool {
    pub fn new(device: &ash::Device, queue_family_idx: u32, pool_flags: vk::CommandPoolCreateFlags) -> Self {
        let pool = create_pool(device, queue_family_idx, pool_flags);
        let buffers = create_buffers(pool, device, config::MAX_FRAMES_IN_FLIGHT as u32);

        Self {
            pool,
            cmd_buffers: buffers,
        }
    }

    pub fn default_for_graphics(device: &VulkanLogicDevice) -> Self {
        VulkanCommandPool::new(&device.device, device.queues.graphics.family_idx, vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
    }

    pub fn default_for_transfer(device: &VulkanLogicDevice) -> Self {
        VulkanCommandPool::new(&device.device, device.queues.transfer.family_idx, vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER | vk::CommandPoolCreateFlags::TRANSIENT)
    }

    pub fn get_buffer(&self, idx: usize) -> vk::CommandBuffer {
        self.cmd_buffers[idx]
    }
}

impl VulkanCommandPool {
    // TODO: impl Drop
    pub fn drop_manual(&self, device: &ash::Device) {
        println!("> dropping CommandPool...");

        unsafe {
            // destroys all associated command buffers
            device.destroy_command_pool(self.pool, None);
        }
    }
}


// TODO: error handling
fn create_pool(device: &ash::Device, queue_family_idx: u32, flags: vk::CommandPoolCreateFlags) -> vk::CommandPool {
    let pool_info = vk::CommandPoolCreateInfo::builder()
        .flags(flags)
        .queue_family_index(queue_family_idx)
        .build();

    unsafe { device.create_command_pool(&pool_info, None).unwrap() }
}

// TODO: error handling
fn create_buffers(pool: vk::CommandPool, device: &ash::Device, buffer_count: u32) -> Vec<vk::CommandBuffer> {
    let alloc_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(buffer_count)
        .build();

    unsafe { device.allocate_command_buffers(&alloc_info).unwrap() }
}


impl VulkanCommandPool {
    // TODO: return safe handle
    pub fn begin_single_time_commands(&self, device: &ash::Device) -> vk::CommandBuffer {
        let alloc_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next: ptr::null(),
            command_pool: self.pool,
            level: vk::CommandBufferLevel::PRIMARY,
            command_buffer_count: 1,
        };

        let cmd_buffer = unsafe {
            device.allocate_command_buffers(&alloc_info)
                .expect("failed to create single-time-command-buffer")
                [0]
        };

        let begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: ptr::null(),
            flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
            p_inheritance_info: ptr::null(),
        };

        unsafe {
            device.begin_command_buffer(cmd_buffer, &begin_info)
                .expect("failed to begin command-buffer");
        }

        cmd_buffer
    }

    pub fn end_single_time_commands(&self, device: &ash::Device, cmd_buffer: vk::CommandBuffer, queue: vk::Queue) {
        unsafe {
            device.end_command_buffer(cmd_buffer)
                .expect("failed to end single-time-command-buffer");
        }

        let submit_info = vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: ptr::null(),
            wait_semaphore_count: 0,
            p_wait_semaphores: ptr::null(),
            p_wait_dst_stage_mask: ptr::null(),
            command_buffer_count: 1,
            p_command_buffers: &cmd_buffer,
            signal_semaphore_count: 0,
            p_signal_semaphores: ptr::null(),
        };

        unsafe {
            device.queue_submit(queue, &[submit_info], vk::Fence::null())
                .expect("failed to submit single-time-command-buffer");
            device.queue_wait_idle(queue).unwrap();
            device.free_command_buffers(self.pool, &[cmd_buffer]);
        }
    }
}






impl VulkanCommandPool {
    // TODO: error handling
    pub fn reset_cmd_buffer(&self, device: &ash::Device, idx: usize) {
        unsafe {
            device.reset_command_buffer(self.cmd_buffers[idx], vk::CommandBufferResetFlags::empty()) .unwrap()
        }
    }

}
