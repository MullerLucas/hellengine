use ash::vk;
use crate::vulkan::{VulkanBuffer, VulkanCore};

#[derive(Clone)]
pub struct VulkanCamera {
    pub view: glam::Mat4,
    pub proj: glam::Mat4,
    pub view_proj: glam::Mat4,
}

impl VulkanCamera {
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

impl VulkanCamera {
    pub fn device_size() -> vk::DeviceSize {
        std::mem::size_of::<Self>() as vk::DeviceSize
    }

    pub fn update_view_proj(&mut self) {
        self.view_proj = self.proj * self.view;
    }

    // TODO: error handling
    pub fn update_uniform_buffer(&mut self, core: &VulkanCore, delta_time: f32, buffer: &VulkanBuffer) {
        let device = &core.device.device;

        static mut POS: glam::Vec3 = glam::Vec3::new(0.0, 0.0, 0.0);
        unsafe {
            POS.x += delta_time * 10.0;
        }

        self.update_view_proj();

        let buff_size = std::mem::size_of::<VulkanCamera>() as u64;
        // let uniform_buffer = &self.uniform_buffers_per_frame[img_idx];


        unsafe {
            let data_ptr = device.map_memory(buffer.mem, 0, buff_size, vk::MemoryMapFlags::empty())
                .unwrap() as *mut VulkanCamera;
            data_ptr.copy_from_nonoverlapping(self, 1);
            device.unmap_memory(buffer.mem);
        }
    }
}