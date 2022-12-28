use std::sync::Arc;

use hell_common::window::{HellSurfaceInfo, HellWindowExtent};
use hell_error::HellResult;
use hell_resources::ResourceManager;

use crate::camera::HellCamera;
use crate::render_types::RenderPackage;
use crate::shader::SpriteShaderSceneData;
use crate::vulkan::primitives::VulkanSwapchain;
use crate::vulkan::{VulkanBackend, VulkanContext, VulkanFrameData};



pub struct HellRendererInfo {
    pub max_frames_in_flight: usize,
    pub surface_info: HellSurfaceInfo,
    pub window_extent: HellWindowExtent,
}

pub struct HellRenderer {
    info: HellRendererInfo,
    backend: VulkanBackend,

    frame_idx: usize,
    camera: HellCamera,
}

impl HellRenderer {
    pub fn new(info: HellRendererInfo) -> HellResult<Self> {
        let ctx = Arc::new(VulkanContext::new(&info.surface_info)?);
        let swapchain = VulkanSwapchain::new(&ctx, info.window_extent)?;
        let aspect_ratio = swapchain.aspect_ratio();
        let backend = VulkanBackend::new(ctx, swapchain)?;

        let camera = HellCamera::new(aspect_ratio);

        Ok(Self {
            info,
            backend,
            frame_idx: 0,
            camera,
        })
    }
}

impl HellRenderer {
    pub fn frame_idx(&self) -> usize {
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
        // let shaders = resource_manager.unique_shader_keys();
        // self.backend.create_shaders(shaders, resource_manager)?;
        self.backend.create_textures(resource_manager)?;

        Ok(())
    }

    pub fn draw_frame(&mut self, delta_time: f32, scene_data: &SpriteShaderSceneData, render_pkg: &RenderPackage, resources: &ResourceManager) -> HellResult<bool> {
        self.backend.update_world_shader(self.camera.clone(), scene_data, &render_pkg.world)?;
        self.backend.update_font_shader (self.camera.clone(), &render_pkg.ui)?;

        let is_resized = self.backend.draw_frame(delta_time, render_pkg, resources)?;

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

    pub fn get_core(&self) -> &VulkanContext {
        &self.backend.ctx
    }
}

