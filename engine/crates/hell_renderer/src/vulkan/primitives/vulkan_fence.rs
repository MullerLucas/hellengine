use ash::vk;
use hell_error::{HellResult, ErrToHellErr};

use crate::vulkan::VulkanCtxRef;



#[derive(Debug, Clone)]
pub struct VulkanFence {
    ctx: VulkanCtxRef,
    handle: vk::Fence,
}

impl Drop for VulkanFence {
    fn drop(&mut self) {
        unsafe {
            self.ctx.device.device.destroy_fence(self.handle, None);
        }
    }
}

impl VulkanFence {
    pub fn new(ctx: &VulkanCtxRef, create_info: &vk::FenceCreateInfo) -> HellResult<Self> {
        let handle = unsafe { ctx.device.device.create_fence(create_info, None)? };

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
            Ok(self.ctx.device.device.wait_for_fences(
                &[self.handle()],
                true,
                timeout,
            )?)
        }
    }

    pub fn reset_fence(&self) -> HellResult<()> {
        unsafe { self.ctx.device.device.reset_fences(&[self.handle()]).to_render_hell_err() }
    }
}
