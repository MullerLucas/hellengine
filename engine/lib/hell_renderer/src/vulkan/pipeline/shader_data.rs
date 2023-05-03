use ash::vk;
use hell_core::error::HellResult;
use crate::vulkan::{VulkanContextRef, primitives::{VulkanBuffer, VulkanCommands}, Vertex3D, Vertex2D};


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

static QUAD_VERTS_4D: &[Vertex3D] = &[
    // Top-Left
    Vertex3D::from_arrays([-0.5,  0.5,  0.0], [0.0, 0.0]),
    // Bottom-Left
    Vertex3D::from_arrays([-0.5, -0.5,  0.0], [0.0, 1.0]),
    // Bottom-Right
    Vertex3D::from_arrays([ 0.5, -0.5,  0.0], [1.0, 1.0]),
    // Top-Right
    Vertex3D::from_arrays([ 0.5,  0.5,  0.0], [1.0, 0.0]),
];

static QUAD_VERTS_2D: &[Vertex2D] = &[
    // Top-Left
    Vertex2D::from_arrays([-0.5,  0.5], [0.0, 0.0]),
    // Bottom-Left
    Vertex2D::from_arrays([-0.5, -0.5], [0.0, 1.0]),
    // Bottom-Right
    Vertex2D::from_arrays([ 0.5, -0.5], [1.0, 1.0]),
    // Top-Right
    Vertex2D::from_arrays([ 0.5,  0.5], [1.0, 0.0]),
];

static QUAD_INDICES: &[u32] = &[
    0, 1, 2,
    2, 3, 0,
];



// ----------------------------------------------------------------------------
// mesh
// ----------------------------------------------------------------------------

pub type VulkanWorldMesh = VulkanMesh<Vertex3D>;
pub type VulkanUiMesh    = VulkanMesh<Vertex2D>;

#[derive(Debug)]
pub struct VulkanMesh<T> {
    pub vertices: Vec<T>,
    pub indices: Vec<u32>,

    pub vertex_buffer: VulkanBuffer,
    pub index_buffer: VulkanBuffer,
}

impl<T> VulkanMesh<T> {
    pub const INDEX_TYPE: vk::IndexType = vk::IndexType::UINT32;

    pub fn indices_count(&self) -> usize {
        self.indices.len()
    }
}

impl VulkanMesh<Vertex3D> {
    pub fn new_quad_3d(ctx: &VulkanContextRef, cmds: &VulkanCommands) -> HellResult<Self> {
        Ok(Self {
            vertices: QUAD_VERTS_4D.to_vec(),
            indices: QUAD_INDICES.to_vec(),

            vertex_buffer: VulkanBuffer::from_vertices(ctx, cmds, QUAD_VERTS_4D)?,
            index_buffer: VulkanBuffer::from_indices(ctx, cmds, QUAD_INDICES)?,
        })
    }
}

impl VulkanMesh<Vertex2D> {
    pub fn new_quad_2d(ctx: &VulkanContextRef, cmds: &VulkanCommands) -> HellResult<Self> {
        Ok(Self {
            vertices: QUAD_VERTS_2D.to_vec(),
            indices: QUAD_INDICES.to_vec(),

            vertex_buffer: VulkanBuffer::from_vertices(ctx, cmds, QUAD_VERTS_4D)?,
            index_buffer: VulkanBuffer::from_indices(ctx, cmds, QUAD_INDICES)?,
        })
    }
}






// ----------------------------------------------------------------------------
// push-constants
// ----------------------------------------------------------------------------

#[derive(Debug)]
pub struct MeshPushConstants {
    pub model: glam::Mat4,
}
