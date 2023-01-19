use ash::vk;


pub const APP_NAME: &str = "hellengine";
pub const ENGINE_NAME: &str = "hellengine";
pub const ENGINE_VERSION: u32 = 1;


// -----------------------------------------------------------------------------
// rendering
// -----------------------------------------------------------------------------

pub const ENABLE_VALIDATION_LAYERS: bool = true;
pub const VALIDATION_LAYER_NAMES: &[&str] = &[
    "VK_LAYER_KHRONOS_validation"
];

pub const DEVICE_EXTENSION_NAMES: &[&str] = &[
    "VK_KHR_swapchain",
];

pub const FRAMES_IN_FLIGHT: usize = 3;
pub const FALLBACK_PRESENT_MODE: vk::PresentModeKHR = vk::PresentModeKHR::FIFO;

pub const ENABLE_SAMPLE_SHADING: bool = true;


pub const FRAME_BUFFER_LAYER_COUNT: u32 = 1;

pub const CLEAR_COLOR: [f32; 4] = [0.3, 0.2, 0.8, 1.0];


// -----------------------------------------------------------------------------
// resources
// -----------------------------------------------------------------------------
pub const IMG_FLIP_V: bool = false;
pub const IMG_FLIP_H: bool = false;

pub const SPRITE_SHADER_KEY:  &str = "sprite";
pub const SPRITE_SHADER_PATH: &str = "shaders/sprite";
pub const TEST_SHADER_KEY:    &str = "test";
pub const TEST_SHADER_PATH:   &str = "shaders/test";


// guaranteed by the spec -> 128 Bytes for push constants
// VULKAN_PUSH_CONSTANT_STRIDE = 128;

// Some nvidia devices have a 256 byte alignment requirement ?

// UBO - 1024
// IMAGE - SAMPLERS 4096

pub const VULKAN_UBO_DESCRIPTOR_COUNT: usize = 1024;
pub const VULKAN_SAMPLER_DESCRIPTOR_COUNT: usize = 4096;
pub const VULKAN_STORAGE_UBO_DESCRIPTOR_COUNT: usize = 1024;

pub const VULKAN_GUARANTEED_PUSH_CONSTANT_STRIDE: usize = 128;
pub const VULKAN_NVIDIA_REQUIRED_ALIGNMENT: usize= 256;

// TODO: think about values
// maximum number of descriptor sets that may be allocated
pub const MAX_DESCRIPTOR_SET_COUNT: usize = 1024;

pub const VULKAN_MAX_MATERIAL_COUNT: usize = 1024;
pub const VULKAN_MAX_SAMPLERS_PER_SHADER: usize = 16;

pub const VULKAN_SHADER_MAX_STAGES: usize =  8;
pub const VULKAN_SHADER_MAX_GLOBAL_TEXTURES: usize =  31;
pub const VULKAN_SHADER_MAX_INSTANCE_TEXTURES: usize =  31;
pub const VULKAN_SHADER_MAX_ATTRIBUTES: usize = 16;
pub const VULKAN_SHADER_MAX_PUSH_CONSTANTS: usize = 16;

