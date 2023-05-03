use ash::vk;
use hell_core::error::{HellResult, ErrToHellErr};
use crate::vulkan::VulkanContextRef;



#[derive(Debug, Clone)]
pub struct VulkanFence {
    ctx: VulkanContextRef,
    handle: vk::Fence,
}

impl Drop for VulkanFence {
    fn drop(&mut self) {
        unsafe {
            self.ctx.device.handle.destroy_fence(self.handle, None);
        }
    }
}

impl VulkanFence {
    pub fn new(ctx: &VulkanContextRef, create_info: &vk::FenceCreateInfo) -> HellResult<Self> {
        let handle = unsafe { ctx.device.handle.create_fence(create_info, None)? };

        Ok(Self {
            ctx: ctx.clone(),
            handle,
        })
    }

    pub fn handle(&self) -> vk::Fence {
        self.handle
    }

    pub fn wait_for_fence(&self, timeout: u64) -> HellResult<()> {
        unsafe {
            Ok(self.ctx.device.handle.wait_for_fences(
                &[self.handle()],
                true,
                timeout,
            )?)
        }
    }

    pub fn reset_fence(&self) -> HellResult<()> {
        unsafe { self.ctx.device.handle.reset_fences(&[self.handle()]).to_render_hell_err() }
    }
}
