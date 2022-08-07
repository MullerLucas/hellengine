use ash::prelude::VkResult;
pub use ash::vk;

use super::config;
use super::swapchain::Swapchain;



pub struct FrameData {
    pub img_available_sem: [vk::Semaphore; 1],
    pub render_finished_sem: [vk::Semaphore; 1],
    pub in_flight_fence: [vk::Fence; 1],
    pub wait_stages: [vk::PipelineStageFlags; 1],
}

impl FrameData {
    pub fn new(device: &ash::Device) -> Self {
        let semaphore_info = vk::SemaphoreCreateInfo::default();

        let fence_info = vk::FenceCreateInfo::builder()
            // create fence in signaled state so the first call to draw_frame works
            .flags(vk::FenceCreateFlags::SIGNALED)
            .build();

        // TODO: error handling
        let img_available_sem = unsafe { device.create_semaphore(&semaphore_info, None).unwrap() };
        let render_finished_sem = unsafe { device.create_semaphore(&semaphore_info, None).unwrap() };
        let in_flight_fence = unsafe { device.create_fence(&fence_info, None).unwrap() };

        Self {
            img_available_sem: [img_available_sem],
            render_finished_sem: [render_finished_sem],
            in_flight_fence: [in_flight_fence],
            wait_stages: [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT]
        }
    }

    pub fn create_for_frames(device: &ash::Device) -> Vec<Self> {
        (0..config::MAX_FRAMES_IN_FLIGHT).into_iter()
            .map(|_| FrameData::new(device))
            .collect()
    }
}

impl FrameData {
    // TODO: impl Drop
    pub fn drop_manual(&self, device: &ash::Device) {
        println!("> dropping FrameData...");

        unsafe {
            device.destroy_semaphore(self.img_available_sem[0], None);
            device.destroy_semaphore(self.render_finished_sem[0], None);
            device.destroy_fence(self.in_flight_fence[0], None);
        }
    }
}

impl FrameData {
    // TODO: error handling
    pub fn wait_for_in_flight(&self, device: &ash::Device) {
        unsafe {
            device.wait_for_fences(
                &self.in_flight_fence,
                true,
                u64::MAX    // ^= "don't time out"
            ) .unwrap();
        }
    }

    // TODO: error handling
    pub fn reset_in_flight_fence(&self, device: &ash::Device) {
        unsafe {
            device.reset_fences(&self.in_flight_fence).unwrap();
        }
    }

    // TODO: error handling
    pub fn submit_queue(&self, device: &ash::Device, queue: vk::Queue, cmd_buffers: &[vk::CommandBuffer]) {
        let submit_info = [
            vk::SubmitInfo::builder()
                .wait_semaphores(&self.img_available_sem)
                .wait_dst_stage_mask(&self.wait_stages)
                .signal_semaphores(&self.render_finished_sem)
                .command_buffers(cmd_buffers)
                .build()
        ];

        unsafe {
            device.queue_submit(queue, &submit_info, self.in_flight_fence[0]).unwrap();
        }
    }

    // TODO: error handling
    pub fn present_queue(&self, queue: vk::Queue, swapchain: &Swapchain, img_indices: &[u32]) -> VkResult<bool> {
        let s = &[swapchain.vk_swapchain];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(&self.render_finished_sem)
            .swapchains(s)
            .image_indices(img_indices)
            .build();

        unsafe {
            swapchain.swapchain_loader.queue_present(queue, &present_info)
        }
    }
}
