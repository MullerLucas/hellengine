use std::ptr;

use ash::vk;

use super::buffer::{VulkanBuffer, VulkanUniformData};
use super::config;
use super::logic_device::VulkanLogicDevice;
use super::pipeline::VulkanGraphicsPipeline;
use super::render_pass::VulkanRenderPassData;
use super::vulkan_core::VulkanCore;




pub struct VulkanCommandPool {
    pub pool: vk::CommandPool,
    pub cmd_buffers: Vec<vk::CommandBuffer>
}

impl VulkanCommandPool {
    pub fn new(device: &ash::Device, queue_family_idx: u32, pool_flags: vk::CommandPoolCreateFlags) -> Self {
        let pool = create_pool(device, queue_family_idx, pool_flags);
        let buffers = create_buffers(pool, device, config::MAX_FRAMES_IN_FLIGHT);

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

    pub fn get_buffer_for_frame(&self, frame_idx: usize) -> vk::CommandBuffer {
        self.cmd_buffers[frame_idx]
    }
}

impl VulkanCommandPool {
    // TODO: impl Drop
    pub fn drop_manual(&mut self, device: &ash::Device) {
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
    pub fn reset_cmd_buffer(&self, device: &ash::Device, curr_frame: usize) {
        unsafe {
            device.reset_command_buffer(self.cmd_buffers[curr_frame], vk::CommandBufferResetFlags::empty()) .unwrap()
        }
    }

    // TODO: error handling
    #[allow(clippy::too_many_arguments)]
    pub fn record_cmd_buffer(
        &self,
        core: &VulkanCore,
        pipeline: &VulkanGraphicsPipeline,
        render_pass_data: &VulkanRenderPassData,
        frame_idx: usize,
        swqp_img_idx: usize,
        index_count: u32,
        vertex_buffer: &VulkanBuffer,
        index_buffer: &VulkanBuffer,
        uniform_data: &VulkanUniformData,
    ) {

        let begin_info = vk::CommandBufferBeginInfo::default();
        let cmd_buffer = self.cmd_buffers[frame_idx];
        let device = &core.device.device;

        unsafe { device.begin_command_buffer(cmd_buffer, &begin_info).unwrap(); }

        // one clear-color per attachment with load-op-clear - order should be identical
        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue { float32: config::CLEAR_COLOR }
            },
            vk::ClearValue {
                // range of depth: [0, 1]
                depth_stencil: vk::ClearDepthStencilValue{ depth: 1.0, stencil: 0 }
            }
        ];

        let render_area = vk::Rect2D {
            offset: vk::Offset2D::default(),
            extent: core.swapchain.extent
        };

        let render_pass_info = vk::RenderPassBeginInfo::builder()
            .render_pass(render_pass_data.render_pass.render_pass)
            .framebuffer(render_pass_data.framebuffer.buffer_at(swqp_img_idx))
            .clear_values(&clear_values)
            .render_area(render_area)
            .build();


        // recorrd commands
        unsafe {
            device.cmd_begin_render_pass(cmd_buffer, &render_pass_info, vk::SubpassContents::INLINE);

            device.cmd_bind_pipeline(cmd_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline.pipeline);
            let vertex_buffers = [vertex_buffer.buffer];
            device.cmd_bind_vertex_buffers(cmd_buffer, 0, &vertex_buffers, &[0]);
            device.cmd_bind_index_buffer(cmd_buffer, index_buffer.buffer, 0, config::INDEX_TYPE);

            let descriptor_set = [uniform_data.descriptor_pool.sets[frame_idx]];
            device.cmd_bind_descriptor_sets(cmd_buffer, vk::PipelineBindPoint::GRAPHICS, pipeline.pipeline_layout, 0, &descriptor_set, &[]);

            device.cmd_draw_indexed(cmd_buffer, index_count, 1, 0, 0, 0);

            device.cmd_end_render_pass(cmd_buffer);
        }

        unsafe {
            device.end_command_buffer(cmd_buffer).unwrap();
        }
    }
}
