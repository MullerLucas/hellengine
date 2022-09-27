use ash::prelude::VkResult;
use ash::vk;
use crate::shared::camera::Camera;
use crate::vulkan::{VulkanBuffer, VulkanCore};

use super::RenderData;



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

impl VulkanUboData for Camera {
    fn device_size() -> vk::DeviceSize {
        std::mem::size_of::<Self>() as vk::DeviceSize
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
// scene data
// ----------------------------------------------------------------------------

#[repr(C)]
pub struct ObjectData {
    pub model: glam::Mat4,
}

impl VulkanUboData for ObjectData {
    fn device_size() -> vk::DeviceSize {
        std::mem::size_of::<Self>() as vk::DeviceSize
    }
}

impl ObjectData {
    pub const MAX_OBJ_COUNT: u64 = 10000;

    pub fn total_size() -> vk::DeviceSize {
        (Self::device_size() *  Self::MAX_OBJ_COUNT) as vk::DeviceSize
    }

    pub fn update_storage_buffer(core: &VulkanCore, buffer: &VulkanBuffer, render_data: &RenderData) -> VkResult<()>{
        let object_data: Vec<_> = render_data.iter()
            .map(|r| ObjectData {
                model: r.transform.create_model_mat()
            })
            .collect();

        unsafe {
            // TODO: try to write diretly into the buffer
            buffer.upload_data_storage_buffer(&core.device.device, object_data.as_ptr(), object_data.len())?;
        }

        Ok(())
    }
}



// ----------------------------------------------------------------------------
// push-constants
// ----------------------------------------------------------------------------

#[derive(Debug)]
pub struct MeshPushConstants {
    pub model: glam::Mat4,
}



// ----------------------------------------------------------------------------
// utils
// ----------------------------------------------------------------------------

pub fn calculate_aligned_size(min_alignment: u64, orig_size: u64) -> u64 {
    if min_alignment == 0 { return orig_size; }
    (orig_size + min_alignment - 1) & !(min_alignment - 1)
}

