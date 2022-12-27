use ash::vk;

pub trait VulkanUboData {
    fn device_size() -> vk::DeviceSize;

    fn padded_device_size(min_ubo_alignment: u64) -> vk::DeviceSize {
        calculate_aligned_size(min_ubo_alignment, Self::device_size())
    }
}

pub fn calculate_aligned_size(min_alignment: u64, orig_size: u64) -> u64 {
    if min_alignment == 0 { return orig_size; }
    (orig_size + min_alignment - 1) & !(min_alignment - 1)
}
