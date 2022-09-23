use ash::prelude::VkResult;
use ash::vk;
use crate::vulkan::{VulkanBuffer, VulkanCore};



// ----------------------------------------------------------------------------
// vulkan ubo data
// ----------------------------------------------------------------------------

pub trait VulkanUboData {
    fn device_size() -> vk::DeviceSize;

    fn padded_device_size(min_ubo_alignment: u64) -> vk::DeviceSize {
        calculate_aligned_size(min_ubo_alignment, Self::device_size() as u64)
    }

}


// ----------------------------------------------------------------------------
// camera data
// ----------------------------------------------------------------------------

#[derive(Clone)]
#[repr(C)]
pub struct CameraData {
    pub view: glam::Mat4,
    pub proj: glam::Mat4,
    pub view_proj: glam::Mat4,
}

impl VulkanUboData for CameraData {
    fn device_size() -> vk::DeviceSize {
        std::mem::size_of::<Self>() as vk::DeviceSize
    }
}

impl CameraData {
    pub fn new(core: &VulkanCore) -> Self {
        let aspect_ratio = core.swapchain.aspect_ratio();

        let view = glam::Mat4::look_at_rh(glam::Vec3::new(0.0, 0.0, 2.0), glam::Vec3::new(0.0, 0.0, 0.0), glam::Vec3::new(0.0, 1.0, 0.0));
        let proj = glam::Mat4::perspective_rh(90.0, aspect_ratio, 0.1, 10.0);
        let view_proj = view * proj;

        Self {
            view, proj, view_proj
        }
    }
}

impl CameraData {
    pub fn update_view_proj(&mut self) {
        self.view_proj = self.proj * self.view;
    }

    // TODO: error handling
    pub fn update_uniform_buffer(&mut self, core: &VulkanCore, delta_time: f32, buffer: &VulkanBuffer) -> VkResult<()> {
        static mut POS: glam::Vec3 = glam::Vec3::new(0.0, 0.0, 0.0);
        unsafe { POS.x += delta_time * 10.0; }
        self.update_view_proj();

        buffer.upload_data_buffer(&core.device.device, self)
    }
}


// ----------------------------------------------------------------------------
// scene data
// ----------------------------------------------------------------------------

#[repr(C)]
pub struct SceneData {
    pub tint: glam::Vec4,
    pub sun_color: glam::Vec4,
    pub sun_direction: glam::Vec4,
}

impl VulkanUboData for SceneData {
    fn device_size() -> vk::DeviceSize {
        std::mem::size_of::<Self>() as vk::DeviceSize
    }
}

impl Default for SceneData {
    fn default() -> Self {
        Self {
            tint: glam::Vec4::new(1.0, 1.0, 1.0, 1.0),
            sun_color: glam::Vec4::new(1.0, 1.0, 1.0, 1.0),
            sun_direction: glam::Vec4::new(1.0, 1.0, 1.0, 1.0),
        }
    }
}

impl SceneData {
    pub fn total_size(min_ubo_alignment: u64, frame_count: u64) -> vk::DeviceSize {
        Self::padded_device_size(min_ubo_alignment) * frame_count
    }

    pub fn update_uniform_buffer(&mut self, core: &VulkanCore, buffer: &VulkanBuffer, frame_idx: u64) -> VkResult<()> {
        let min_ubo_alignment = core.phys_device.device_props.limits.min_uniform_buffer_offset_alignment;

        let time_raw = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH).unwrap()
            .as_secs_f64();

        let time = (time_raw / 2.0 % 100_000.0) as f32;

        self.tint = glam::Vec4::new(
            time.sin(),
            time.cos(),
            time.tan(),
            1.0
        );

        buffer.upload_data_buffer_array(&core.device.device, min_ubo_alignment, self, frame_idx)
    }
}



// ----------------------------------------------------------------------------
// utils
// ----------------------------------------------------------------------------

pub fn calculate_aligned_size(min_alignment: u64, orig_size: u64) -> u64 {
    if min_alignment == 0 { return orig_size; }
    (orig_size + min_alignment - 1) & !(min_alignment - 1)
}

