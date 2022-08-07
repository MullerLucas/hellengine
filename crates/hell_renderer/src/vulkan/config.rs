use ash::vk;





pub const APP_NAME: &str = "hellengine";
pub const ENGINE_NAME: &str = "hellengine";
pub const ENGINE_VERSION: u32 = 1;
pub const API_VERSION: u32 = vk::API_VERSION_1_3;

pub const ENABLE_VALIDATION_LAYERS: bool = true;
pub const VALIDATION_LAYER_NAMES: &[&str] = &[
    "VK_LAYER_KHRONOS_validation"
];

pub const DEVICE_EXTENSION_NAMES: &[&str] = &[
    "VK_KHR_swapchain"
];

pub const VERT_SHADER_PATH: &str = "shaders/sprite.vert.spv";
pub const FRAG_SHADER_PATH: &str = "shaders/sprite.frag.spv";

pub const MAX_FRAMES_IN_FLIGHT: u32 = 2;

pub const INDEX_TYPE: vk::IndexType = vk::IndexType::UINT32;

pub const ENABLE_SAMPLE_SHADING: bool = true;


pub const FRAME_BUFFER_LAYER_COUNT: u32 = 1;
