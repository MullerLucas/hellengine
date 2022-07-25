use std::ptr;

use ash::vk;

use super::logic_device::VulkanLogicDevice;



pub struct VulkanCommands {
    graphics_pool: vk::CommandPool,
    transfer_pool: vk::CommandPool,
}

impl VulkanCommands {
    pub fn new(device: &VulkanLogicDevice) -> Self {

        let graphics_pool = create_pool(
            &device.device,
            device.queues.graphics_idx,
            vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER
        );

        let transfer_pool = create_pool(
            &device.device,
            device.queues.graphics_idx,
            vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER | vk::CommandPoolCreateFlags::TRANSIENT
        );

        Self {
            graphics_pool,
            transfer_pool,
        }
    }
}


fn create_pool(
    device: &ash::Device,
    queue_family_idx: u32,
    flags: vk::CommandPoolCreateFlags,
    ) -> vk::CommandPool {

    let pool_info = vk::CommandPoolCreateInfo {
        s_type: vk::StructureType::COMMAND_POOL_CREATE_INFO,
        p_next: ptr::null(),
        flags,
        queue_family_index: queue_family_idx,
    };

    unsafe {
        device
            .create_command_pool(&pool_info, None)
            .expect("failed to create command pool")
    }
}
