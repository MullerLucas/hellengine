use ash::vk;
use hell_error::HellResult;

use crate::vulkan::VulkanCtxRef;




pub struct VulkanSemaphore {
    ctx: VulkanCtxRef,
    handle: vk::Semaphore,
}

impl Drop for VulkanSemaphore {
    fn drop(&mut self) {
        unsafe { self.ctx.device.device.destroy_semaphore(self.handle, None); }
    }
}

impl VulkanSemaphore {
    pub fn new(ctx: &VulkanCtxRef, create_info: &vk::SemaphoreCreateInfo) -> HellResult<Self> {
        let handle = unsafe { ctx.device.device.create_semaphore(&create_info, None)? };

        Ok(Self {
            ctx: ctx.clone(),
            handle,
        })
    }

    pub fn handle(&self) -> vk::Semaphore {
        self.handle
    }
}

