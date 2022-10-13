use ash::vk;
use crate::shared::render_data::{SceneData, CameraData, ObjectData};




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

impl VulkanUboData for CameraData {
    fn device_size() -> vk::DeviceSize {
        std::mem::size_of::<Self>() as vk::DeviceSize
    }
}




// ----------------------------------------------------------------------------
// scene data
// ----------------------------------------------------------------------------

impl VulkanUboData for SceneData {
    fn device_size() -> vk::DeviceSize {
        std::mem::size_of::<Self>() as vk::DeviceSize
    }
}

impl SceneData {
    pub fn total_size(min_ubo_alignment: u64, frame_count: u64) -> vk::DeviceSize {
        Self::padded_device_size(min_ubo_alignment) * frame_count
    }
}



// ----------------------------------------------------------------------------
// scene data
// ----------------------------------------------------------------------------

impl VulkanUboData for ObjectData {
    fn device_size() -> vk::DeviceSize {
        std::mem::size_of::<Self>() as vk::DeviceSize
    }
}

impl ObjectData {
    pub fn total_size() -> vk::DeviceSize {
        (Self::device_size() *  Self::MAX_OBJ_COUNT) as vk::DeviceSize
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

