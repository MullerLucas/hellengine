use std::sync::Arc;

use hell_common::window::HellSurfaceInfo;
use hell_error::HellResult;

use super::command_buffer::VulkanCommandPool;
use super::debugging::VulkanDebugData;
use super::instance::VulkanInstance;
use super::logic_device::VulkanLogicDevice;
use super::phys_device::VulkanPhysDevice;
use super::surface::VulkanSurface;
use hell_core::config;



pub type VulkanCtxRef = Arc<VulkanCtx>;

pub struct VulkanCtx {
    pub debug_data: VulkanDebugData,
    pub surface: VulkanSurface,

    pub phys_device: VulkanPhysDevice,
    pub device: VulkanLogicDevice,

    // pub swapchain: VulkanSwapchain,
    pub graphics_cmd_pool: VulkanCommandPool,
    pub transfer_cmd_pool: VulkanCommandPool,

    pub instance: VulkanInstance,
}

impl std::fmt::Debug for VulkanCtx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // f.debug_struct("VulkanCtx")
        //     .field("debug_data", &self.debug_data)
        //     .field("surface", &self.surface)
        //     .field("phys_device", &self.phys_device)
        //     .field("device", &self.device)
        //     .field("graphics_cmd_pool", &self.graphics_cmd_pool)
        //     .field("transfer_cmd_pool", &self.transfer_cmd_pool)
        //     .field("instance", &self.instance).finish()

        write!(f, "VulkanCtx")
    }
}

impl VulkanCtx {
    pub fn new(surface_info: &HellSurfaceInfo) -> HellResult<Self> {
        let instance = VulkanInstance::new(config::APP_NAME)?;

        let debug_data = VulkanDebugData::new(&instance.entry, &instance.instance);

        let surface = VulkanSurface::new(&instance.entry, &instance.instance, surface_info)?;
        let phys_device = VulkanPhysDevice::pick_phys_device(&instance.instance, &surface)?;
        let device = VulkanLogicDevice::new(&instance.instance, &phys_device)?;

        let graphics_cmd_pool = VulkanCommandPool::default_for_graphics(&device)?;
        let transfer_cmd_pool = VulkanCommandPool::default_for_transfer(&device)?;

        Ok(Self {
            instance,

            surface,

            phys_device,
            device,

            graphics_cmd_pool,
            transfer_cmd_pool,

            debug_data,
        })
    }

    pub fn wait_device_idle(&self) -> HellResult<()> {
        println!("> waiting for the device to be idle...");
        self.device.wait_idle()?;
        println!("> done waiting for the device to be idle...");

        Ok(())
    }
}

impl Drop for VulkanCtx {
    fn drop(&mut self) {
        println!("> dropping Core");
        let device = &self.device.device;

        self.graphics_cmd_pool.drop_manual(device);
        self.transfer_cmd_pool.drop_manual(device);
    }
}
