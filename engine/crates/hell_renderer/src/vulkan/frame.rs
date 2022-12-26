use std::array;

use crate::error::err_invalid_frame_idx;
use crate::shared::collections::PerFrame;
use crate::vulkan::VulkanCommandPool;
use ash::vk;
use hell_error::HellResult;

use super::VulkanCtxRef;
use super::command_buffer::VulkanCommandBuffer;
use super::primitives::{VulkanSemaphore, VulkanFence};


pub struct VulkanFrameData {
    img_available_sem: PerFrame<VulkanSemaphore>,
    render_finished_sem: PerFrame<VulkanSemaphore>,
    in_flight_fences: PerFrame<VulkanFence>,

    pub wait_stages: [vk::PipelineStageFlags; 1], // same for each frame
    pub graphics_cmd_pools: PerFrame<VulkanCommandPool>,

}

impl VulkanFrameData {
    pub fn new(ctx: &VulkanCtxRef) -> HellResult<Self> {
        let semaphore_info = vk::SemaphoreCreateInfo::default();

        let fence_info = vk::FenceCreateInfo::builder()
            // create fence in signaled state so the first call to draw_frame works
            .flags(vk::FenceCreateFlags::SIGNALED)
            .build();

        // TODO: error handling
        let img_available_sem   = array::from_fn(|_| VulkanSemaphore::new(ctx, &semaphore_info).unwrap());
        let render_finished_sem = array::from_fn(|_| VulkanSemaphore::new(ctx, &semaphore_info).unwrap());
        let in_flight_fences    = array::from_fn(|_| VulkanFence::new(ctx, &fence_info).unwrap());
        let graphics_cmd_pools  = array::from_fn(|_| VulkanCommandPool::default_for_graphics(ctx).unwrap());

        Ok(Self {
            img_available_sem,
            render_finished_sem,
            in_flight_fences,
            wait_stages: [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT],
            graphics_cmd_pools,
        })
    }
}

impl VulkanFrameData {
    pub fn in_flight_fence(&self, frame_idx: usize) -> &VulkanFence {
        &self.in_flight_fences[frame_idx]
    }

    pub fn img_available_sem(&self, frame_idx: usize) -> &VulkanSemaphore {
        &self.img_available_sem[frame_idx]
    }

    pub fn img_render_finished_sem(&self, frame_idx: usize) -> &VulkanSemaphore {
        &self.render_finished_sem[frame_idx]
    }

}

impl VulkanFrameData {
    pub fn get_cmd_buffer(&self, frame_idx: usize) -> HellResult<VulkanCommandBuffer> {
        Ok(
            self.graphics_cmd_pools
                .get(frame_idx).ok_or_else(|| err_invalid_frame_idx(frame_idx))?
                .get_buffer(0)
        )
    }
}
