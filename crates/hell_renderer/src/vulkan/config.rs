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
    "VK_KHR_swapchain",
];

pub const VERT_SHADER_PATH: &str = "shaders/sprite.vert.spv";
pub const FRAG_SHADER_PATH: &str = "shaders/sprite.frag.spv";

pub const MAX_FRAMES_IN_FLIGHT: u32 = 3;

pub const DYNAMIC_UNIFORM_DESCRIPTOR_COUNT: u32 = 10;
pub const DYNAMIC_STORAGE_DESCRIPTOR_COUNT: u32 = 10;
pub const TEXTURE_DESCRIPTOR_COUNT: u32 = 10;

pub const MAX_DESCRIPTOR_SET_COUNT: u32 = 10;

pub const ENABLE_SAMPLE_SHADING: bool = true;


pub const FRAME_BUFFER_LAYER_COUNT: u32 = 1;

pub const CLEAR_COLOR: [f32; 4] = [0.1, 0.1, 0.1, 1.0];

pub const TEXTURE_0_PATH: &str = "assets/example_1/texture_0.jpg";
pub const TEXTURE_2_PATH: &str = "assets/example_1/texture_1.jpg";
pub const TEXTURE_1_PATH: &str = "assets/example_1/texture_2.jpg";
