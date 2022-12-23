use ash::vk;
use hell_error::HellResult;
use crate::{shared::render_data::{SceneData, ObjectData}, render_data::GlobalUniformObject, vulkan::{Vertex, VulkanCtxRef, buffer::VulkanBuffer, command_buffer::VulkanCommands}};


/// Vulkan:
///      -1
///      |
/// -1 ----- +1
///      |
///      +1

/// Hell:
///      +1
///      |
/// -1 ----- +1
///      |
///      -1

static QUAD_VERTS: &[Vertex] = &[
    // VULKAN:
    // // Top-Left
    // Vertex::from_arrays([-0.5, -0.5,  0.0, 1.0], [1.0, 0.0, 0.0, 1.0], [0.0, 0.0]),
    // // Bottom-Left
    // Vertex::from_arrays([-0.5,  0.5,  0.0, 1.0], [0.0, 1.0, 0.0, 1.0], [0.0, 1.0]),
    // // Bottom-Right
    // Vertex::from_arrays([ 0.5,  0.5,  0.0, 1.0], [0.0, 0.0, 1.0, 1.0], [1.0, 1.0]),
    // // Top-Right
    // Vertex::from_arrays([ 0.5, -0.5,  0.0, 1.0], [1.0, 1.0, 1.0, 1.0], [1.0, 0.0]),

    // HELL:
    // Top-Left
    Vertex::from_arrays([-0.5,  0.5,  0.0, 1.0], [1.0, 0.0, 0.0, 1.0], [0.0, 0.0]),
    // Bottom-Left
    Vertex::from_arrays([-0.5, -0.5,  0.0, 1.0], [0.0, 1.0, 0.0, 1.0], [0.0, 1.0]),
    // Bottom-Right
    Vertex::from_arrays([ 0.5, -0.5,  0.0, 1.0], [0.0, 0.0, 1.0, 1.0], [1.0, 1.0]),
    // Top-Right
    Vertex::from_arrays([ 0.5,  0.5,  0.0, 1.0], [1.0, 1.0, 1.0, 1.0], [1.0, 0.0]),
];

static QUAD_INDICES: &[u32] = &[
    0, 1, 2,
    2, 3, 0,
];



// ----------------------------------------------------------------------------
// mesh
// ----------------------------------------------------------------------------

#[derive(Debug)]
pub struct VulkanMesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,

    pub vertex_buffer: VulkanBuffer,
    pub index_buffer: VulkanBuffer,
}

impl VulkanMesh {
    pub const INDEX_TYPE: vk::IndexType = vk::IndexType::UINT32;

    pub fn new_quad(ctx: &VulkanCtxRef, cmds: &VulkanCommands) -> HellResult<Self> {
        Ok(Self {
            vertices: QUAD_VERTS.to_vec(),
            indices: QUAD_INDICES.to_vec(),

            vertex_buffer: VulkanBuffer::from_vertices(&ctx, cmds, QUAD_VERTS)?,
            index_buffer: VulkanBuffer::from_indices(&ctx, cmds, QUAD_INDICES)?,
        })
    }

    pub fn indices_count(&self) -> usize {
        self.indices.len()
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

impl VulkanUboData for GlobalUniformObject {
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
// utils
// ----------------------------------------------------------------------------

pub fn calculate_aligned_size(min_alignment: u64, orig_size: u64) -> u64 {
    if min_alignment == 0 { return orig_size; }
    (orig_size + min_alignment - 1) & !(min_alignment - 1)
}
