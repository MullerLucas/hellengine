use std::sync::Arc;
use hell_common::window::HellSurfaceInfo;
use hell_error::HellResult;
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

    pub instance: VulkanInstance,
}

impl std::fmt::Debug for VulkanCtx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
