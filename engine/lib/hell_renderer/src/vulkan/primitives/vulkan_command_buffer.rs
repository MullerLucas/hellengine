use std::ptr;
use ash::vk;
use hell_core::error::{HellResult, ErrToHellErr};
use crate::{vulkan::VulkanContextRef, config};



// ----------------------------------------------------------------------------
// command-pools
// ----------------------------------------------------------------------------

pub struct VulkanCommands {
    pub graphics_pool: VulkanCommandPool,
    pub transfer_pool: VulkanCommandPool,
}

impl VulkanCommands {
    pub fn new(ctx: &VulkanContextRef) -> HellResult<Self> {
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

    #[inline]
    pub fn begin_cmd_buffer(&self, ctx: &VulkanContextRef, begin_info: vk::CommandBufferBeginInfo) -> HellResult<()> {
        unsafe { ctx.device.handle.begin_command_buffer(self.handle(), &begin_info).to_render_hell_err() }
    }

    #[inline]
    pub fn reset_cmd_buffer(&self, ctx: &VulkanContextRef) -> HellResult<()> {
        unsafe { ctx.device.handle.reset_command_buffer(self.handle(), vk::CommandBufferResetFlags::empty()).to_render_hell_err() }
    }

    #[inline]
    pub fn end_command_buffer(&self, ctx: &VulkanContextRef) -> HellResult<()> {
        unsafe { ctx.device.handle.end_command_buffer(self.handle()).to_render_hell_err() }
    }
}

impl<'a> VulkanCommandBuffer<'a> {
    #[inline]
    pub fn cmd_set_viewport(&self, ctx: &VulkanContextRef, first_viewport: u32, viewports: &[vk::Viewport]) {
        unsafe { ctx.device.handle.cmd_set_viewport(self.handle(), first_viewport, viewports); }
    }

    #[inline]
    pub fn cmd_set_scissor(&self, ctx: &VulkanContextRef, first_scissor: u32, scissors: &[vk::Rect2D]) {
        unsafe { ctx.device.handle.cmd_set_scissor(self.handle(), first_scissor, scissors); }
    }

    #[inline]
    pub fn cmd_begin_render_pass(&self, ctx: &VulkanContextRef, create_info: &vk::RenderPassBeginInfo, contents: vk::SubpassContents) {
        unsafe { ctx.device.handle.cmd_begin_render_pass(self.handle(), create_info, contents); }
    }

    #[inline]
    pub fn cmd_end_render_pass(&self, ctx: &VulkanContextRef) {
        unsafe { ctx.device.handle.cmd_end_render_pass(self.handle()); }
    }

    #[inline]
    pub fn cmd_bind_descriptor_sets(&self, ctx: &VulkanContextRef, pipeline_bind_point: vk::PipelineBindPoint, layout: vk::PipelineLayout, first_set: u32, descriptor_sets: &[vk::DescriptorSet], dynamic_offsets: &[u32]) {
        unsafe { ctx.device.handle.cmd_bind_descriptor_sets(self.handle(), pipeline_bind_point, layout, first_set, descriptor_sets, dynamic_offsets); }
    }

    #[inline]
    pub fn cmd_bind_pipeline(&self, ctx: &VulkanContextRef, pipeline_bind_point: vk::PipelineBindPoint, pipeline: vk::Pipeline) {
        unsafe { ctx.device.handle.cmd_bind_pipeline(self.handle(), pipeline_bind_point, pipeline); }
    }

    #[inline]
    pub fn cmd_bind_vertex_buffers(&self, ctx: &VulkanContextRef, first_binding: u32, buffers: &[vk::Buffer], offsets: &[vk::DeviceSize]) {
        unsafe { ctx.device.handle.cmd_bind_vertex_buffers(self.handle(), first_binding, buffers, offsets); }
    }

    #[inline]
    pub fn cmd_bind_index_buffer(&self, ctx: &VulkanContextRef, buffer: vk::Buffer, offset: vk::DeviceSize, index_type: vk::IndexType) {
        unsafe { ctx.device.handle.cmd_bind_index_buffer(self.handle(), buffer, offset, index_type); }
    }

    #[inline]
    pub fn cmd_push_constants(&self, ctx: &VulkanContextRef, layout: vk::PipelineLayout, stage_flags: vk::ShaderStageFlags, offset: usize, constants: &[u8]) {
        unsafe { ctx.device.handle.cmd_push_constants(self.handle(), layout, stage_flags, offset as u32, constants); }
    }

    #[inline]
    pub fn cmd_push_constants_slice<T>(&self, ctx: &VulkanContextRef, layout: vk::PipelineLayout, stage_flags: vk::ShaderStageFlags, offset: usize, constants: &[T]) {
        unsafe {
            let push_const_bytes = std::slice::from_raw_parts(
                constants.as_ptr() as *const u8,
                std::mem::size_of::<T>() * constants.len()
            );

            ctx.device.handle.cmd_push_constants(self.handle(), layout, stage_flags, offset as u32, push_const_bytes);
        }
    }

    #[inline]
    pub fn cmd_draw_indexed(&self, ctx: &VulkanContextRef, index_count: u32, instance_count: u32, first_index: u32, vertex_offset: i32, first_instance: u32) {
        unsafe { ctx.device.handle.cmd_draw_indexed(self.handle(), index_count, instance_count, first_index, vertex_offset, first_instance); }
    }
}

// ----------------------------------------------------------------------------
// command-pool
// ----------------------------------------------------------------------------

pub struct VulkanCommandPool {
    ctx: VulkanContextRef,
    pub pool: vk::CommandPool,
    frame_buffers: Vec<vk::CommandBuffer>,
}

impl Drop for VulkanCommandPool {
    fn drop(&mut self) {
        println!("> dropping CommandPool...");

        unsafe {
            let device = &self.ctx.device.handle;
            // destroys all associated command buffers
            device.destroy_command_pool(self.pool, None);
        }
    }
}

impl VulkanCommandPool {
    pub fn new(ctx: &VulkanContextRef, queue_family_idx: u32, pool_flags: vk::CommandPoolCreateFlags) -> HellResult<Self> {
        let pool = create_pool(&ctx.device.handle, queue_family_idx, pool_flags)?;
        let frame_buffers = Self::create_multiple_buffers(ctx, pool, config::FRAMES_IN_FLIGHT as u32)?;

        Ok(Self {
            ctx: ctx.clone(),
            pool,
            frame_buffers,
        })
    }

    pub fn default_for_graphics(ctx: &VulkanContextRef) -> HellResult<Self> {
        VulkanCommandPool::new(ctx, ctx.device.queues.graphics.family_idx, vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
    }

    pub fn default_for_transfer(ctx: &VulkanContextRef) -> HellResult<Self> {
        VulkanCommandPool::new(ctx, ctx.device.queues.transfer.family_idx, vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER | vk::CommandPoolCreateFlags::TRANSIENT)
    }

    pub fn get_buffer(&self, idx: usize) -> VulkanCommandBuffer {
        VulkanCommandBuffer::new(
            &self.frame_buffers[idx]
        )
    }

    fn create_multiple_buffers(ctx: &VulkanContextRef, pool: vk::CommandPool, count: u32) -> HellResult<Vec<vk::CommandBuffer>> {
        let alloc_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(count)
            .build();

        Ok(unsafe { ctx.device.handle.allocate_command_buffers(&alloc_info) }?)
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
