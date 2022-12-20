use hell_common::window::{HellSurfaceInfo, HellWindowExtent};
use hell_error::{HellResult, ErrToHellErr, HellErrorKind};
use hell_resources::ResourceManager;

use crate::render_data::ObjectData;
use crate::shared::render_data::{CameraData, SceneData};
use crate::vulkan::{VulkanBackend, VulkanCore, VulkanFrameData, RenderData};



pub struct HellRendererInfo {
    pub max_frames_in_flight: usize,
    pub surface_info: HellSurfaceInfo,
    pub window_extent: HellWindowExtent,
}

pub struct HellRenderer {
    info: HellRendererInfo,
    backend: VulkanBackend,
    camera: CameraData,

    frame_idx: usize,
}

impl HellRenderer {
    pub fn new(info: HellRendererInfo) -> HellResult<Self> {
        let core = VulkanCore::new(&info.surface_info, &info.window_extent)?;
        let aspect_ratio = core.swapchain.aspect_ratio();
        let backend = VulkanBackend::new(core)?;
        let camera = CameraData::new(aspect_ratio);

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
    pub fn wait_idle(&self) -> HellResult<()> {
        self.backend.wait_idle()
    }

    pub fn handle_window_changed(&mut self, window_extent: HellWindowExtent) -> HellResult<()> {
        self.info.window_extent = window_extent;
        self.backend.on_window_changed(self.info.window_extent)
    }

    pub fn prepare_renderer(&mut self, resource_manager: &ResourceManager) -> HellResult<()> {
        let shaders = resource_manager.unique_shader_keys();
        self.backend.create_pipelines(shaders)?;
        self.backend.upload_resources(resource_manager)
    }

    pub fn draw_frame(&mut self, delta_time: f32, render_data: &RenderData, resources: &ResourceManager) -> HellResult<bool> {
        self.update_camera(delta_time)?;

        let is_resized = self.backend.draw_frame(delta_time, render_data, resources)?;

        self.increment_frame_idx();
        Ok(is_resized)
    }

    pub fn update_scene_buffer(&self, scene_data: &SceneData) -> HellResult<()> {
        let buffer = self.backend.gpu_resource_manager.get_scene_buffer();
        let min_ubo_alignment = self.backend.core.phys_device.device_props.limits.min_uniform_buffer_offset_alignment;
        buffer.upload_data_buffer_array(&self.backend.core.device.device, min_ubo_alignment, scene_data, self.frame_idx)
            .to_hell_err(HellErrorKind::RenderError)
    }

    pub fn update_object_buffer(&self, render_data: &RenderData) -> HellResult<()> {
        let buffer = self.backend.gpu_resource_manager.get_object_buffer(self.frame_idx)?;

        let object_data: Vec<_> = render_data.iter()
            .map(|r| ObjectData {
                model: r.transform.create_model_mat()
            })
            .collect();

        unsafe {
            // TODO: try to write diretly into the buffer
            buffer.upload_data_storage_buffer(&self.backend.core.device.device, object_data.as_ptr(), object_data.len())
        }
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

