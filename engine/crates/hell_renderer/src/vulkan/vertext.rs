use std::mem;

use ash::vk;
use memoffset::offset_of;

// ----------------------------------------------------------------------------
// Vertex 3D
// ----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Vertex3D {
    pub pos: glam::Vec3,
    pub tex_coord: glam::Vec2,
}

impl Vertex3D {
    pub const fn from_arrays(pos: [f32; 3], tex_coord: [f32; 2]) -> Self {
        Self {
            pos: glam::Vec3::from_array(pos),
            tex_coord: glam::Vec2::from_array(tex_coord)
        }
    }
}

impl Vertex3D {
    pub const fn get_binding_desc() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription {
            binding: 0,
            stride: mem::size_of::<Vertex3D>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }
    }

    pub fn get_attribute_desc() -> Vec<vk::VertexInputAttributeDescription> {
        vec![
            vk::VertexInputAttributeDescription { location: 0, binding: 0, format: vk::Format::R32G32B32A32_SFLOAT, offset: offset_of!(Self, pos) as u32 },
            vk::VertexInputAttributeDescription { location: 1, binding: 0, format: vk::Format::R32G32_SFLOAT,       offset: offset_of!(Self, tex_coord) as u32 },
        ]
    }

    pub const fn structure_size() -> vk::DeviceSize {
        mem::size_of::<Self>() as vk::DeviceSize
    }
}



// ----------------------------------------------------------------------------
// Vertex 2D
// ----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct Vertex2D {
    pub pos: glam::Vec2,
    pub tex_coord: glam::Vec2,
}

impl Vertex2D {
    pub const fn from_arrays(pos: [f32; 2], tex_coord: [f32; 2]) -> Self {
        Self {
            pos: glam::Vec2::from_array(pos),
            tex_coord: glam::Vec2::from_array(tex_coord)
        }
    }

    pub const fn get_binding_desc() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription {
            binding: 0,
            stride: mem::size_of::<Vertex3D>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }
    }

    pub fn get_attribute_desc() -> Vec<vk::VertexInputAttributeDescription> {
        vec![
            vk::VertexInputAttributeDescription { location: 0, binding: 0, format: vk::Format::R32G32_SFLOAT, offset: offset_of!(Self, pos) as u32 },
            vk::VertexInputAttributeDescription { location: 1, binding: 0, format: vk::Format::R32G32_SFLOAT, offset: offset_of!(Self, tex_coord) as u32 },
        ]
    }

    pub const fn structure_size() -> vk::DeviceSize {
        mem::size_of::<Self>() as vk::DeviceSize
    }
}
