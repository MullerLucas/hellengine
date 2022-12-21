use ash::prelude::VkResult;
use ash::vk;








// ----------------------------------------------------------------------------
// descriptor-pool
// ----------------------------------------------------------------------------

pub struct VulkanDescriptorSetGroup {
    pub layout: vk::DescriptorSetLayout,
    pub sets: Vec<Vec<vk::DescriptorSet>>, // per frame
}

impl VulkanDescriptorSetGroup {
    pub fn new(layout: vk::DescriptorSetLayout) -> Self {
        Self {
            layout,
            sets: vec![]
        }
    }

    pub fn new_global_group(device: &ash::Device) -> VkResult<Self> {
        let layout = Self::create_global_set_layout(device)?;

        Ok(Self::new(layout))
    }

    pub fn new_object_group(device: &ash::Device) -> VkResult<Self> {
        let layout = Self::create_object_set_layout(device)?;

        Ok(Self::new(layout))
    }

    pub fn new_material_group(device: &ash::Device) -> VkResult<Self> {
        let layout = Self::create_material_set_layout(device)?;

        Ok(Self::new(layout))
    }

    fn create_global_set_layout(device: &ash::Device) -> VkResult<vk::DescriptorSetLayout> {
        let bindings = [
            // Global-Uniform
            vk::DescriptorSetLayoutBinding::builder()
                .binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX)
                .build(),
            // Scene-Data
            vk::DescriptorSetLayoutBinding::builder()
                .binding(1)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
                .build()
        ];

        Self::create_descriptor_set_layout(device, &bindings)
    }

    fn create_object_set_layout(device: &ash::Device) -> VkResult<vk::DescriptorSetLayout> {
        let bindings = [
            // Per-Object-Data
            vk::DescriptorSetLayoutBinding::builder()
                .binding(0)
                .descriptor_count(1)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .stage_flags(vk::ShaderStageFlags::VERTEX)
                .build()
        ];

        Self::create_descriptor_set_layout(device, &bindings)
    }

    fn create_material_set_layout(device: &ash::Device) -> VkResult<vk::DescriptorSetLayout> {
        let bindings = [
            // texture_sampler
            vk::DescriptorSetLayoutBinding::builder()
                .binding(0)
                .descriptor_count(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                .build()
        ];

        Self::create_descriptor_set_layout(device, &bindings)
    }

    fn create_descriptor_set_layout(device: &ash::Device, bindings: &[vk::DescriptorSetLayoutBinding]) -> VkResult<vk::DescriptorSetLayout> {
        let layout_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(bindings)
            .build();

        Ok(unsafe {
            device.create_descriptor_set_layout(&layout_info, None)?
        })
    }
}

impl VulkanDescriptorSetGroup {
    pub fn drop_manual(&self, device: &ash::Device) {
        println!("> dropping VulkanDescriptorSetLayoutGroup...");

        unsafe {
            device.destroy_descriptor_set_layout(self.layout, None);
        }
    }
}

impl VulkanDescriptorSetGroup {
    pub fn set_count(&self) -> usize {
        self.sets.len()
    }
}





// ----------------------------------------------------------------------------
// descriptor
// ----------------------------------------------------------------------------


