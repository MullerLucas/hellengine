use ash::prelude::VkResult;
use hell_common::window::{HellSurfaceInfo, HellWindowExtent};

use super::command_buffer::VulkanCommandPool;
use super::debugging::VulkanDebugData;
use super::instance::VulkanInstance;
use super::logic_device::VulkanLogicDevice;
use super::phys_device::VulkanPhysDevice;
use super::surface::VulkanSurface;
use super::swapchain::VulkanSwapchain;
use super::config;




pub struct VulkanCore {
    pub debug_data: VulkanDebugData,

    pub surface: VulkanSurface,

    pub phys_device: VulkanPhysDevice,
    pub device: VulkanLogicDevice,

    pub swapchain: VulkanSwapchain,

    pub graphics_cmd_pool: VulkanCommandPool,
    pub transfer_cmd_pool: VulkanCommandPool,

    pub instance: VulkanInstance,
}

impl VulkanCore {
    pub fn new(surface_info: &HellSurfaceInfo, windwow_extent: &HellWindowExtent) -> VkResult<Self> {
        let instance = VulkanInstance::new(config::APP_NAME);

        let debug_data = VulkanDebugData::new(&instance.entry, &instance.instance);

        let surface = VulkanSurface::new(&instance.entry, &instance.instance, surface_info);
        let phys_device = VulkanPhysDevice::pick_phys_device(&instance.instance, &surface);
        let device = VulkanLogicDevice::new(&instance.instance, &phys_device);

        let graphics_cmd_pool = VulkanCommandPool::default_for_graphics(&device);
        let transfer_cmd_pool = VulkanCommandPool::default_for_transfer(&device);

        let swapchain = VulkanSwapchain::new(&instance.instance, &phys_device, &device, &surface, windwow_extent.width, windwow_extent.height);


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

        self.swapchain.drop_manual(&self.device.device);

        let swapchain = VulkanSwapchain::new(&self.instance.instance, &self.phys_device, &self.device, &self.surface, window_extent.width, window_extent.height);
        self.swapchain = swapchain;
    }

    pub fn wait_device_idle(&self) {
        println!("> waiting for the device to be idle...");
        self.device.wait_idle();
        println!("> done waiting for the device to be idle...");
    }
}

impl Drop for VulkanCore {
    fn drop(&mut self) {
        println!("> dropping Core");
        let device = &self.device.device;

        self.graphics_cmd_pool.drop_manual(device);
        self.transfer_cmd_pool.drop_manual(device);
        self.swapchain.drop_manual(device);
    }
}
