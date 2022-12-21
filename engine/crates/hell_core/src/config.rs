pub const APP_NAME: &str = "hellengine";
pub const ENGINE_NAME: &str = "hellengine";
pub const ENGINE_VERSION: u32 = 1;
// pub const API_VERSION: u32 = vk::API_VERSION_1_3;

pub const ENABLE_VALIDATION_LAYERS: bool = true;
pub const VALIDATION_LAYER_NAMES: &[&str] = &[
    "VK_LAYER_KHRONOS_validation"
];

pub const DEVICE_EXTENSION_NAMES: &[&str] = &[
    "VK_KHR_swapchain",
];

pub const VERT_SHADER_PATH: &str = "shaders/sprite.vert.spv";
pub const FRAG_SHADER_PATH: &str = "shaders/sprite.frag.spv";

pub const FRAMES_IN_FLIGHT: usize = 3;

pub const DYNAMIC_UNIFORM_DESCRIPTOR_COUNT: u32 = 10;
pub const DYNAMIC_STORAGE_DESCRIPTOR_COUNT: u32 = 10;
pub const TEXTURE_DESCRIPTOR_COUNT: u32 = 10;

pub const MAX_DESCRIPTOR_SET_COUNT: u32 = 10;

pub const ENABLE_SAMPLE_SHADING: bool = true;


pub const FRAME_BUFFER_LAYER_COUNT: u32 = 1;

pub const CLEAR_COLOR: [f32; 4] = [0.3, 0.2, 0.8, 1.0];

pub const IMG_FLIP_V: bool = false;
pub const IMG_FLIP_H: bool = false;
