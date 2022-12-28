use ash::vk;

use crate::shader::base_shader::CameraUniform;

use super::shader_utils::VulkanUboData;

impl VulkanUboData for CameraUniform {
    fn device_size() -> vk::DeviceSize {
        std::mem::size_of::<Self>() as vk::DeviceSize
    }
}
