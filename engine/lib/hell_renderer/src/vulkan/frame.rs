use std::array;

use crate::config;
use crate::render_types::PerFrame;
use ash::vk;
use hell_core::error::HellResult;

use super::VulkanContextRef;
use super::primitives::{VulkanSemaphore, VulkanFence, VulkanCommandPool, VulkanCommandBuffer};




pub struct VulkanFrame {
    #[allow(dead_code)] ctx: VulkanContextRef,
    frame_idx: usize,

    img_available_sem: PerFrame<VulkanSemaphore>,
    render_finished_sem: PerFrame<VulkanSemaphore>,
    in_flight_fences: PerFrame<VulkanFence>,
    wait_stages: vk::PipelineStageFlags, // same for each frame

    gfx_cmd_pools: PerFrame<VulkanCommandPool>,
}

impl VulkanFrame {
    pub fn new(ctx: &VulkanContextRef) -> HellResult<Self> {
        let semaphore_info = vk::SemaphoreCreateInfo::default();

        let fence_info = vk::FenceCreateInfo::builder()
            // create fence in signaled state so the first call to draw_frame works
            .flags(vk::FenceCreateFlags::SIGNALED)
            .build();

        // TODO: error handling
        let img_available_sem   = array::from_fn(|_| VulkanSemaphore::new(ctx, &semaphore_info).unwrap());
        let render_finished_sem = array::from_fn(|_| VulkanSemaphore::new(ctx, &semaphore_info).unwrap());
        let in_flight_fences    = array::from_fn(|_| VulkanFence::new(ctx, &fence_info).unwrap());
        let gfx_cmd_pools  = array::from_fn(|_| VulkanCommandPool::default_for_graphics(ctx).unwrap());
        let wait_stages = vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT;

        Ok(Self {
            ctx: ctx.clone(),
            frame_idx: 0,

            img_available_sem,
            render_finished_sem,
            in_flight_fences,
            wait_stages,
            gfx_cmd_pools,
        })
    }
}

impl VulkanFrame {
    pub fn begin_frame(&self) {
        // let cmd_buff = self.gfx_cmd_buffer();
        // let cmd_buff_begin_info = vk::CommandBufferBeginInfo::default();
        // cmd_buff.begin_cmd_buffer(&self.ctx, begin_info)?;
    }

    pub fn end_frame(&mut self) {
        self.frame_idx = (self.frame_idx + 1) % config::FRAMES_IN_FLIGHT;
    }
}

impl VulkanFrame {
    pub fn idx(&self) -> usize {
        self.frame_idx
    }

    pub fn wait_stages(&self) -> vk::PipelineStageFlags {
        self.wait_stages
    }

    pub fn in_flight_fence(&self) -> &VulkanFence {
        &self.in_flight_fences[self.frame_idx]
    }

    pub fn img_available_sem(&self) -> &VulkanSemaphore {
        &self.img_available_sem[self.frame_idx]
    }

    pub fn img_render_finished_sem(&self) -> &VulkanSemaphore {
        &self.render_finished_sem[self.frame_idx]
    }

    pub fn gfx_cmd_buffer(&self) -> VulkanCommandBuffer {
        self.gfx_cmd_pools[self.frame_idx]
            .get_buffer(0)
    }
}
