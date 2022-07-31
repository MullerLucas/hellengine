use ash::vk;

use super::vertext::Vertex;

pub const WINDOW_WIDTH: u32 = 800;
pub const WINDOW_HEIGHT: u32 = 600;

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

pub const VERT_SHADER_PATH: &str = "shaders/spv/triangle_vert.spv";
pub const FRAG_SHADER_PATH: &str = "shaders/spv/triangle_frag.spv";

pub const MAX_FRAMES_IN_FLIGHT: u32 = 2;


pub static VERTICES: &[Vertex] = &[
    Vertex::from_arrays([-0.5, -0.5,  0.0, 1.0], [1.0, 1.0, 1.0, 1.0], [1.0, 0.0]),
    Vertex::from_arrays([ 0.5, -0.5,  0.0, 1.0], [1.0, 1.0, 1.0, 1.0], [0.0, 0.0]),
    Vertex::from_arrays([ 0.5,  0.5,  0.0, 1.0], [1.0, 1.0, 1.0, 1.0], [0.0, 1.0]),
    Vertex::from_arrays([-0.5,  0.5,  0.0, 1.0], [1.0, 1.0, 1.0, 1.0], [1.0, 1.0]),
    // Vertex::from_arrays([-0.5, -0.5, -0.5, 1.0], [1.0, 1.0, 1.0, 1.0], [1.0, 0.0]),
    // Vertex::from_arrays([ 0.5, -0.5, -0.5, 1.0], [1.0, 1.0, 1.0, 1.0], [0.0, 0.0]),
    // Vertex::from_arrays([ 0.5,  0.5, -0.5, 1.0], [1.0, 1.0, 1.0, 1.0], [0.0, 1.0]),
    // Vertex::from_arrays([-0.5,  0.5, -0.5, 1.0], [1.0, 1.0, 1.0, 1.0], [1.0, 1.0]),
];

pub const INDEX_TYPE: vk::IndexType = vk::IndexType::UINT32;
pub static INDICES: &[u32] = &[     // u32 is also possible
    0, 1, 2,
    2, 3, 0,
    // 4, 5, 6,
    // 6, 7, 4
];

// pub static _VERTICES: &[Vertex] = &[
//     Vertex { pos: [-0.5, -0.5, 0.0, 1.0], color: [1.0, 1.0, 1.0, 1.0], tex_coord: [1.0, 0.0]},
//     Vertex { pos: [ 0.5, -0.5, 0.0, 1.0], color: [1.0, 1.0, 1.0, 1.0], tex_coord: [0.0, 0.0]},
//     Vertex { pos: [ 0.5,  0.5, 0.0, 1.0], color: [1.0, 1.0, 1.0, 1.0], tex_coord: [0.0, 1.0]},
//     Vertex { pos: [-0.5,  0.5, 0.0, 1.0], color: [1.0, 1.0, 1.0, 1.0], tex_coord: [1.0, 1.0]},
//
//     Vertex { pos: [-0.5, -0.5, -0.5, 1.0], color: [1.0, 1.0, 1.0, 1.0], tex_coord: [1.0, 0.0]},
//     Vertex { pos: [ 0.5, -0.5, -0.5, 1.0], color: [1.0, 1.0, 1.0, 1.0], tex_coord: [0.0, 0.0]},
//     Vertex { pos: [ 0.5,  0.5, -0.5, 1.0], color: [1.0, 1.0, 1.0, 1.0], tex_coord: [0.0, 1.0]},
//     Vertex { pos: [-0.5,  0.5, -0.5, 1.0], color: [1.0, 1.0, 1.0, 1.0], tex_coord: [1.0, 1.0]},
// ];

// pub static INDICES: &[u16] = &[     // u32 is also possible
//     0, 1, 2,
//     2, 3, 0,
//     4, 5, 6,
//     6, 7, 4
// ];

// pub const VIKING_MODEL_PATH: &str = "assets/viking_room/viking_room.obj";
// pub const VIKING_TEXTURE_PATH: &str = "assets/viking_room/viking_room.png";

pub const ENABLE_SAMPLE_SHADING: bool = true;
pub const MIN_SAMPLE_SHADING: f32 = if ENABLE_SAMPLE_SHADING { 0.2 } else { 1.0 };


pub const FRAME_BUFFER_LAYER_COUNT: u32 = 1;
