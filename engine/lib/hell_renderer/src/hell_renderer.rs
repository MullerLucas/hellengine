use std::sync::Arc;

use hell_common::window::{HellSurfaceInfo, HellWindowExtent};
use hell_core::error::HellResult;

use crate::camera::HellCamera;
use crate::render_types::RenderPackage;
use crate::resources::{TextureManager, MaterialManager, ShaderManager, ResourceHandle};
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
    pub sha_man: ShaderManager,
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
        let sha_man = ShaderManager::default();

        Ok(Self {
            info,
            backend,
            camera,

            mat_man,
            tex_man,
            sha_man,
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

    #[allow(unused)]
    pub fn prepare_renderer(&mut self) -> HellResult<()> {
        let sprite_handle = self.acquire_shader("sprite", true)?;
        let sprite_shader = self.sha_man.shader_mut(sprite_handle);
        let player_tex = self.tex_man.acquire_textuer(&self.backend, "player_tex".to_string(), Some("assets/characters/player_char.png".to_string()),   false, false)?;
        let enemy_tex  = self.tex_man.acquire_textuer(&self.backend, "enemy_tex".to_string(),  Some("assets/characters/enemy_t1_char.png".to_string()), false, false)?;
        let ground_tex = self.tex_man.acquire_textuer(&self.backend, "enemy_tex".to_string(),  Some("assets/environment/ground_v1.png".to_string()),    false, false)?;
        let _ = sprite_shader.acquire_instance_resource(&[enemy_tex])?;
        let _ = sprite_shader.acquire_instance_resource(&[player_tex])?;
        let _ = sprite_shader.acquire_instance_resource(&[ground_tex])?;
        // TODO: local instances
        let _ = sprite_shader.acquire_local_resource(&[])?;

        let handle = self.acquire_shader("test", false)?;
        let shader = self.sha_man.shader_mut(handle);
        let tex_1 = self.tex_man.acquire_textuer(&self.backend, "instance_tex_1".to_string(), Some("assets/characters/enemy_t1_char.png".to_string()), false, false)?;
        let tex_2 = self.tex_man.acquire_textuer(&self.backend, "instance_tex_2".to_string(), Some("assets/characters/player_char.png".to_string()), false, false)?;
        let _ = shader.acquire_shared_resource(&[])?;
        let _ = shader.acquire_instance_resource(&[tex_1])?;
        let _ = shader.acquire_instance_resource(&[tex_2])?;
        // TODO: local instances
        let _ = shader.acquire_local_resource(&[])?;

        Ok(())
    }

    pub fn draw_frame(&mut self, delta_time: f32, render_pkg: &RenderPackage) -> HellResult<bool> {
        self.backend.begin_frame()?;
        self.backend.draw_frame(delta_time, render_pkg, &mut self.sha_man, &self.tex_man, &self.camera)?;
        let is_resized = self.backend.end_frame()?;
        Ok(is_resized)
    }
}

impl HellRenderer {
    // TODO: this sux
    pub fn acquire_shader(&mut self, key: &str, is_sprite_shader: bool) -> HellResult<ResourceHandle> {
        let tex = self.tex_man.acquire_textuer(&self.backend, "test_global".to_string(), None, false, false)?;
        let shader = self.sha_man.create_shader(&self.backend, key, tex, is_sprite_shader)?;
        Ok(shader)
    }

    pub fn acquire_material(&mut self, path: impl Into<String>) -> HellResult<ResourceHandle> {
        self.mat_man.acquire_from_file(&self.backend, &mut self.tex_man, path.into())
    }
}
