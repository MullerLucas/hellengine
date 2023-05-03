use ash::vk;
use hell_core::error::HellResult;
use crate::vulkan::VulkanContextRef;




pub struct VulkanSemaphore {
    ctx: VulkanContextRef,
    handle: vk::Semaphore,
}

impl Drop for VulkanSemaphore {
    fn drop(&mut self) {
        unsafe { self.ctx.device.handle.destroy_semaphore(self.handle, None); }
    }
}

impl VulkanSemaphore {
    pub fn new(ctx: &VulkanContextRef, create_info: &vk::SemaphoreCreateInfo) -> HellResult<Self> {
        let handle = unsafe { ctx.device.handle.create_semaphore(create_info, None)? };

        Ok(Self {
            ctx: ctx.clone(),
            handle,
        })
    }

    pub fn handle(&self) -> vk::Semaphore {
        self.handle
    }
}

