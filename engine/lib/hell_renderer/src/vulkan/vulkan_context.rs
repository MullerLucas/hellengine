use std::sync::Arc;
use hell_common::window::HellSurfaceInfo;
use hell_core::error::HellResult;
use crate::config;

use super::debugging::VulkanDebugData;
use super::primitives::{VulkanSurface, VulkanLogicDevice, VulkanPhysDevice, VulkanInstance};





pub type VulkanContextRef = Arc<VulkanContext>;

pub struct VulkanContext {
    pub debug_data: VulkanDebugData,
    pub surface: VulkanSurface,
    pub phys_device: VulkanPhysDevice,
    pub device: VulkanLogicDevice,

    pub instance: VulkanInstance,
}

impl std::fmt::Debug for VulkanContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "VulkanCtx")
    }
}

impl VulkanContext {
    pub fn new(surface_info: &HellSurfaceInfo) -> HellResult<Self> {
        let instance = VulkanInstance::new(config::APP_NAME)?;
        let debug_data = VulkanDebugData::new(&instance.entry, &instance.instance);
        let surface = VulkanSurface::new(&instance.entry, &instance.instance, surface_info)?;
        let phys_device = VulkanPhysDevice::pick_phys_device(&instance.instance, &surface)?;
        let device = VulkanLogicDevice::new(&instance.instance, &phys_device)?;

        Ok(Self {
            instance,
            surface,
            phys_device,
            device,
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
