use std::ptr;
use ash::vk;
use hell_error::{HellResult, ErrToHellErr};
use hell_core::config;
use super::VulkanCtxRef;



// ----------------------------------------------------------------------------
// command-pools
// ----------------------------------------------------------------------------

pub struct VulkanCommands {
    pub graphics_pool: VulkanCommandPool,
    pub transfer_pool: VulkanCommandPool,
}

impl VulkanCommands {
    pub fn new(ctx: &VulkanCtxRef) -> HellResult<Self> {
        let graphics_cmd_pool = VulkanCommandPool::default_for_graphics(ctx)?;
        let transfer_cmd_pool = VulkanCommandPool::default_for_transfer(ctx)?;

        Ok(Self {
            graphics_pool: graphics_cmd_pool,
            transfer_pool: transfer_cmd_pool,
        })
    }
}




// ----------------------------------------------------------------------------
// buffer-handle
// ----------------------------------------------------------------------------

pub struct VulkanCommandBuffer<'a> {
    // NOTE: technically not required to be a reference - but being a reference, it's lifetime is tied to the lifetime of the Command-Pool, which is nice
    handle: &'a vk::CommandBuffer,
}

impl<'a> VulkanCommandBuffer<'a> {
    pub fn new(handle: &'a vk::CommandBuffer) -> Self {
        Self { handle }
    }

    pub fn handle(&self) -> vk::CommandBuffer {
        *self.handle
    }

    pub fn begin_cmd_buffer(&self, ctx: &VulkanCtxRef) -> HellResult<()> {
        let begin_info = vk::CommandBufferBeginInfo::default();
        unsafe { ctx.device.device.begin_command_buffer(*self.handle, &begin_info)? }
        Ok(())
    }

    pub fn cmd_set_viewport(&self, ctx: &VulkanCtxRef, first_viewport: u32, viewports: &[vk::Viewport]) {
        unsafe { ctx.device.device.cmd_set_viewport(self.handle(), first_viewport, viewports); }
    }

    pub fn cmd_set_scissor(&self, ctx: &VulkanCtxRef, first_scissor: u32, scissors: &[vk::Rect2D]) {
        unsafe { ctx.device.device.cmd_set_scissor(self.handle(), first_scissor, scissors); }
    }
}

// ----------------------------------------------------------------------------
// command-pool
// ----------------------------------------------------------------------------

pub struct VulkanCommandPool {
    ctx: VulkanCtxRef,
    pub pool: vk::CommandPool,
    buffers: Vec<vk::CommandBuffer>
}

impl Drop for VulkanCommandPool {
    fn drop(&mut self) {
        println!("> dropping CommandPool...");

        unsafe {
            let device = &self.ctx.device.device;
            // destroys all associated command buffers
            device.destroy_command_pool(self.pool, None);
        }
    }
}

impl VulkanCommandPool {
    pub fn new(ctx: &VulkanCtxRef, queue_family_idx: u32, pool_flags: vk::CommandPoolCreateFlags) -> HellResult<Self> {
        let pool = create_pool(&ctx.device.device, queue_family_idx, pool_flags)?;
        let buffers = Self::create_multiple_buffers(ctx, pool, config::FRAMES_IN_FLIGHT as u32)?;

        Ok(Self {
            ctx: ctx.clone(),
            pool,
            buffers,
        })
    }

    pub fn default_for_graphics(ctx: &VulkanCtxRef) -> HellResult<Self> {
        VulkanCommandPool::new(ctx, ctx.device.queues.graphics.family_idx, vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
    }

    pub fn default_for_transfer(ctx: &VulkanCtxRef) -> HellResult<Self> {
        VulkanCommandPool::new(ctx, ctx.device.queues.transfer.family_idx, vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER | vk::CommandPoolCreateFlags::TRANSIENT)
    }

    pub fn get_buffer(&self, idx: usize) -> VulkanCommandBuffer {
        VulkanCommandBuffer::new(
            &self.buffers[idx]
        )
    }

    fn create_multiple_buffers(ctx: &VulkanCtxRef, pool: vk::CommandPool, count: u32) -> HellResult<Vec<vk::CommandBuffer>> {
        let alloc_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(count)
            .build();

        Ok(unsafe { ctx.device.device.allocate_command_buffers(&alloc_info) }?)
    }
}

fn create_pool(device: &ash::Device, queue_family_idx: u32, flags: vk::CommandPoolCreateFlags) -> HellResult<vk::CommandPool> {
    let pool_info = vk::CommandPoolCreateInfo::builder()
        .flags(flags)
        .queue_family_index(queue_family_idx)
        .build();

    unsafe { device.create_command_pool(&pool_info, None).to_render_hell_err() }
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

    pub fn end_single_time_commands(&self, device: &ash::Device, cmd_buffer: vk::CommandBuffer, queue: vk::Queue) -> HellResult<()>{
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
            device.queue_wait_idle(queue).to_render_hell_err()?;
            device.free_command_buffers(self.pool, &[cmd_buffer]);
        }

        Ok(())
    }
}






impl VulkanCommandPool {
    pub fn reset_cmd_buffer(&self, device: &ash::Device, idx: usize) -> HellResult<()> {
        unsafe {
            device.reset_command_buffer(self.buffers[idx], vk::CommandBufferResetFlags::empty()).to_render_hell_err()
        }
    }

}
