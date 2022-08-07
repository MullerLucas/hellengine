use ash::prelude::VkResult;
use hell_common::window::{HellSurfaceInfo, HellWindowExtent};

use super::command_buffer::CommandPool;
use super::debugging::DebugData;
use super::instance::Instance;
use super::logic_device::LogicDevice;
use super::phys_device::VulkanPhysDevice;
use super::surface::Surface;
use super::swapchain::Swapchain;
use super::config;

use crate::vulkan;





pub struct Core {
    pub debug_data: DebugData,

    pub surface: Surface,

    pub phys_device: VulkanPhysDevice,
    pub device: LogicDevice,

    pub swapchain: Swapchain,

    pub graphics_cmd_pool: CommandPool,
    pub transfer_cmd_pool: CommandPool,

    pub instance: vulkan::Instance,
}

impl Core {
    pub fn new(surface_info: &HellSurfaceInfo, windwow_extent: &HellWindowExtent) -> VkResult<Self> {
        let instance = Instance::new(config::APP_NAME);

        let debug_data = DebugData::new(&instance.entry, &instance.instance);

        let surface = Surface::new(&instance.entry, &instance.instance, surface_info);
        let phys_device = VulkanPhysDevice::pick_phys_device(&instance.instance, &surface);
        let device = LogicDevice::new(&instance.instance, &phys_device);

        let graphics_cmd_pool = CommandPool::default_for_graphics(&device);
        let transfer_cmd_pool = CommandPool::default_for_transfer(&device);

        let swapchain = Swapchain::new(&instance.instance, &phys_device, &device, &surface, windwow_extent.width, windwow_extent.height);


        Ok(Self {
            instance,

            surface,

            phys_device,
            device,

            graphics_cmd_pool,
            transfer_cmd_pool,

            swapchain,

            debug_data,
        })
    }

    pub fn recreate_swapchain(&mut self, window_extent: &HellWindowExtent) {
        println!("> recreating swapchain...");

        self.swapchain.drop_manual(&self.device.vk_device);

        let swapchain = Swapchain::new(&self.instance.instance, &self.phys_device, &self.device, &self.surface, window_extent.width, window_extent.height);
        self.swapchain = swapchain;
    }

    pub fn wait_device_idle(&self) {
        println!("> waiting for the device to be idle...");
        self.device.wait_idle();
        println!("> done waiting for the device to be idle...");
    }

    pub fn phys_device(&self) -> &VulkanPhysDevice {
        &self.phys_device
    }
}

impl Drop for Core {
    fn drop(&mut self) {
        println!("> dropping Core");
        let device = &self.device.vk_device;

        self.graphics_cmd_pool.drop_manual(device);
        self.transfer_cmd_pool.drop_manual(device);
        self.swapchain.drop_manual(device);
    }
}
