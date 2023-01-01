use std::sync::Arc;

use hell_common::window::{HellSurfaceInfo, HellWindowExtent};
use hell_error::HellResult;

use crate::camera::HellCamera;
use crate::render_types::RenderPackage;
use crate::resources::{TextureManager, MaterialManager, ResourceHandle};
use crate::shader::SpriteShaderSceneData;
use crate::vulkan::primitives::VulkanSwapchain;
use crate::vulkan::{VulkanBackend, VulkanContext};



pub struct HellRendererInfo {
    pub max_frames_in_flight: usize,
    pub surface_info: HellSurfaceInfo,
    pub window_extent: HellWindowExtent,
}

pub struct HellRenderer {
    info: HellRendererInfo,
    backend: VulkanBackend,

    // frame_idx: usize,
    camera: HellCamera,

    pub mat_man: MaterialManager,
    pub tex_man: TextureManager,
}

impl HellRenderer {
    pub fn new(info: HellRendererInfo) -> HellResult<Self> {
        let ctx = Arc::new(VulkanContext::new(&info.surface_info)?);
        let swapchain = VulkanSwapchain::new(&ctx, info.window_extent)?;
        let aspect_ratio = swapchain.aspect_ratio();
        let backend = VulkanBackend::new(ctx, swapchain)?;

        let camera = HellCamera::new(aspect_ratio);

        let mat_man = MaterialManager::default();
        let tex_man = TextureManager::default();

        Ok(Self {
            info,
            backend,
            // frame_idx: 0,
            camera,

            mat_man,
            tex_man,
        })
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

    pub fn prepare_renderer(&mut self) -> HellResult<()> {
        self.backend.create_textures(&self.tex_man)
    }

    pub fn draw_frame(&mut self, delta_time: f32, scene_data: &SpriteShaderSceneData, render_pkg: &RenderPackage) -> HellResult<bool> {
        self.backend.update_world_shader(self.camera.clone(), scene_data, &render_pkg.world)?;

        self.backend.begin_frame()?;
        self.backend.draw_frame(delta_time, render_pkg)?;
        let is_resized = self.backend.end_frame()?;

        Ok(is_resized)
    }
}

impl HellRenderer {
    pub fn acquire_material_from_file(&mut self, path: impl Into<String>) -> HellResult<ResourceHandle> {
        self.mat_man.create_from_file(&self.backend, &mut self.tex_man, path.into())
    }
}
