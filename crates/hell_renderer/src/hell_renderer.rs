use hell_common::HellResult;
use hell_common::window::{HellSurfaceInfo, HellWindowExtent};

use crate::shared::camera::Camera;
use crate::vulkan::{VulkanBackend, VulkanCore, VulkanFrameData, RenderData};
use crate::error::vk_to_hell_err;

pub struct HellRendererInfo {
    pub max_frames_in_flight: usize,
    pub surface_info: HellSurfaceInfo,
    pub window_extent: HellWindowExtent,
}

pub struct HellRenderer {
    info: HellRendererInfo,
    backend: VulkanBackend,
    camera: Camera,

    frame_idx: usize,
}

impl HellRenderer {
    pub fn new(info: HellRendererInfo) -> HellResult<Self> {
        let core = VulkanCore::new(&info.surface_info, &info.window_extent).map_err(vk_to_hell_err)?;
        let aspect_ratio = core.swapchain.aspect_ratio();
        let backend = VulkanBackend::new(core);
        let camera = Camera::new(aspect_ratio);

        Ok(Self {
            info,
            backend,
            camera,
            frame_idx: 0,
        })
    }
}

impl HellRenderer {
    pub fn get_frame_idx(&self) -> usize {
        self.frame_idx
    }
}

impl HellRenderer {
    pub fn wait_idle(&self) {
        self.backend.wait_idle()
    }

    pub fn handle_window_changed(&mut self, window_extent: HellWindowExtent) {
        self.info.window_extent = window_extent;
        self.backend.on_window_changed(self.info.window_extent);
    }

    pub fn draw_frame(&mut self, delta_time: f32, render_data: &RenderData) -> HellResult<bool> {
        self.update_camera(delta_time)?;

        let is_resized = self.backend.draw_frame(delta_time, render_data);

        self.increment_frame_idx();
        Ok(is_resized)
    }
}

impl HellRenderer {
    fn increment_frame_idx(&mut self) {
        self.frame_idx = (self.frame_idx + 1) % self.info.max_frames_in_flight;
    }

    fn update_camera(&mut self, delta_time: f32) -> HellResult<()> {
        static mut POS: glam::Vec3 = glam::Vec3::new(0.0, 0.0, 0.0);
        unsafe { POS.x += delta_time * 10.0; }
        self.camera.update_view_proj();

        self.backend.update_camera_buffer(self.frame_idx, &self.camera)?;
        // buffer.upload_data_buffer(&core.device.device, self)

        Ok(())
    }
}

// HACK: remove
impl HellRenderer {
    pub fn get_frame_data(&self) -> &VulkanFrameData {
        &self.backend.frame_data
    }

    pub fn get_core(&self) -> &VulkanCore {
        &self.backend.core
    }
}

