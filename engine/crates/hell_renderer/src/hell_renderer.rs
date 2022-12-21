use hell_common::window::{HellSurfaceInfo, HellWindowExtent};
use hell_error::HellResult;
use hell_resources::ResourceManager;

use crate::render_data::TmpCamera;
use crate::shared::render_data::SceneData;
use crate::vulkan::{VulkanBackend, VulkanCore, VulkanFrameData, RenderData};



pub struct HellRendererInfo {
    pub max_frames_in_flight: usize,
    pub surface_info: HellSurfaceInfo,
    pub window_extent: HellWindowExtent,
}

pub struct HellRenderer {
    info: HellRendererInfo,
    backend: VulkanBackend,

    frame_idx: usize,

    camera: TmpCamera,
}

impl HellRenderer {
    pub fn new(info: HellRendererInfo) -> HellResult<Self> {
        let core = VulkanCore::new(&info.surface_info, &info.window_extent)?;
        let aspect_ratio = core.swapchain.aspect_ratio();
        let backend = VulkanBackend::new(core)?;

        let camera = TmpCamera::new(aspect_ratio);

        Ok(Self {
            info,
            backend,
            frame_idx: 0,
            camera,
        })
    }
}

impl HellRenderer {
    pub fn get_frame_idx(&self) -> usize {
        self.frame_idx
    }
}

impl HellRenderer {
    pub fn wait_idle(&self) -> HellResult<()> {
        self.backend.wait_idle()
    }

    pub fn handle_window_changed(&mut self, window_extent: HellWindowExtent) -> HellResult<()> {
        self.info.window_extent = window_extent;
        self.backend.on_window_changed(self.info.window_extent)
    }

    pub fn prepare_renderer(&mut self, resource_manager: &ResourceManager) -> HellResult<()> {
        let shaders = resource_manager.unique_shader_keys();
        self.backend.create_shaders(shaders, resource_manager)?;
        Ok(())
    }

    pub fn draw_frame(&mut self, delta_time: f32, scene_data: &SceneData, render_data: &RenderData, resources: &ResourceManager) -> HellResult<bool> {
        self.backend.update_global_state(self.camera.clone())?;
        self.backend.update_scene_buffer(scene_data)?;
        self.backend.update_object_buffer(render_data)?;

        let is_resized = self.backend.draw_frame(delta_time, render_data, resources)?;

        self.increment_frame_idx();
        Ok(is_resized)
    }

}

impl HellRenderer {
    fn increment_frame_idx(&mut self) {
        self.frame_idx = (self.frame_idx + 1) % self.info.max_frames_in_flight;
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

