use ash::prelude::VkResult;
pub use ash::vk;
use crate::vulkan::{VulkanBuffer, VulkanCommandPool};

use super::{config, VulkanCore, SceneData, ObjectData};
use super::swapchain::VulkanSwapchain;



pub struct VulkanFrameData {
    pub img_available_semaphors: Vec<[vk::Semaphore; 1]>,
    pub render_finished_semaphors: Vec<[vk::Semaphore; 1]>,
    pub in_flight_fences: Vec<[vk::Fence; 1]>,
    pub wait_stages: [vk::PipelineStageFlags; 1], // same for each frame
    pub graphics_cmd_pools: Vec<VulkanCommandPool>,

    pub scene_ubo: VulkanBuffer, // one ubo for all frames
    pub object_ubos: Vec<VulkanBuffer>,
}

impl VulkanFrameData {
    pub fn new(core: &VulkanCore) -> Self {
        let device = &core.device.device;

        let semaphore_info = vk::SemaphoreCreateInfo::default();

        let fence_info = vk::FenceCreateInfo::builder()
            // create fence in signaled state so the first call to draw_frame works
            .flags(vk::FenceCreateFlags::SIGNALED)
            .build();

        // TODO: error handling
        let img_available_sem: Vec<_> = (0..config::MAX_FRAMES_IN_FLIGHT).into_iter()
            .map(|_| unsafe { [device.create_semaphore(&semaphore_info, None).unwrap()] })
            .collect();
        let render_finished_sem: Vec<_> = (0..config::MAX_FRAMES_IN_FLIGHT).into_iter()
            .map(|_| unsafe { [device.create_semaphore(&semaphore_info, None).unwrap()] })
            .collect();
        let in_flight_fence: Vec<_> = (0..config::MAX_FRAMES_IN_FLIGHT).into_iter()
            .map(|_| unsafe { [device.create_fence(&fence_info, None).unwrap()] })
            .collect();

        let graphics_cmd_pool: Vec<_> = (0..config::MAX_FRAMES_IN_FLIGHT).into_iter()
            .map(|_| VulkanCommandPool::default_for_graphics(&core.device))
            .collect();

        let scene_ubo_size = SceneData::total_size(core.phys_device.device_props.limits.min_uniform_buffer_offset_alignment, config::MAX_FRAMES_IN_FLIGHT as u64);
        let scene_ubo = VulkanBuffer::from_uniform(core, scene_ubo_size);

        let object_ubo_size = ObjectData::total_size();
        let object_ubos: Vec<_> = (0..config::MAX_FRAMES_IN_FLIGHT).into_iter()
            .map(|_| VulkanBuffer::from_storage(core, object_ubo_size))
            .collect();


        Self {
            img_available_semaphors: img_available_sem,
            render_finished_semaphors: render_finished_sem,
            in_flight_fences: in_flight_fence,
            wait_stages: [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT],
            graphics_cmd_pools: graphics_cmd_pool,

            scene_ubo,
            object_ubos,
        }
    }
}

impl VulkanFrameData {
    // TODO: impl Drop
    pub fn drop_manual(&self, device: &ash::Device) {
        println!("> dropping FrameData...");

        self.graphics_cmd_pools.iter().for_each(|p| p.drop_manual(device));
        self.scene_ubo.drop_manual(device);
        self.object_ubos.iter().for_each(|p| p.drop_manual(device));

        unsafe {
            self.img_available_semaphors.iter().for_each(|s| device.destroy_semaphore(s[0], None));
            self.render_finished_semaphors.iter().for_each(|s| device.destroy_semaphore(s[0], None));
            self.in_flight_fences.iter().for_each(|f| device.destroy_fence(f[0], None));
        }
    }
}

impl VulkanFrameData {
    // TODO: error handling
    pub fn wait_for_in_flight(&self, device: &ash::Device, frame_idx: usize) {
        unsafe {
            device.wait_for_fences(
                &self.in_flight_fences[frame_idx],
                true,
                u64::MAX    // ^= "don't time out"
            ) .unwrap();
        }
    }

    // TODO: error handling
    pub fn reset_in_flight_fence(&self, device: &ash::Device, frame_idx: usize) {
        unsafe {
            device.reset_fences(&self.in_flight_fences[frame_idx]).unwrap();
        }
    }

    // TODO: error handling
    pub fn submit_queue(&self, device: &ash::Device, queue: vk::Queue, cmd_buffers: &[vk::CommandBuffer], frame_idx: usize) {
        let submit_info = [
            vk::SubmitInfo::builder()
                .wait_semaphores(&self.img_available_semaphors[frame_idx])
                .wait_dst_stage_mask(&self.wait_stages)
                .signal_semaphores(&self.render_finished_semaphors[frame_idx])
                .command_buffers(cmd_buffers)
                .build()
        ];

        unsafe {
            device.queue_submit(queue, &submit_info, self.in_flight_fences[frame_idx][0]).unwrap();
        }
    }

    // TODO: error handling
    pub fn present_queue(&self, queue: vk::Queue, swapchain: &VulkanSwapchain, img_indices: &[u32], frame_idx: usize) -> VkResult<bool> {
        let s = &[swapchain.vk_swapchain];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&self.render_finished_semaphors[frame_idx])
            .swapchains(s)
            .image_indices(img_indices)
            .build();

        unsafe {
            swapchain.swapchain_loader.queue_present(queue, &present_info)
        }
    }
}

impl VulkanFrameData {
    pub fn get_cmd_buffer(&self, frame_idx: usize) -> vk::CommandBuffer {
        self.graphics_cmd_pools
            .get(frame_idx).unwrap()
            .get_buffer(0)
    }
}
