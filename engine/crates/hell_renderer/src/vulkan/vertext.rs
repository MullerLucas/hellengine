use std::mem;

use ash::vk;
use memoffset::offset_of;

#[derive(Clone)]
pub struct Vertex {
    pub pos: glam::Vec4,
    pub color: glam::Vec4,
    pub tex_coord: glam::Vec2,
}

impl Vertex {
    pub const fn from_arrays(pos: [f32; 4], color: [f32; 4], tex_coord: [f32; 2]) -> Self {
        Self {
            pos: glam::Vec4::from_array(pos),
            color: glam::Vec4::from_array(color),
            tex_coord: glam::Vec2::from_array(tex_coord)
        }
    }
}

impl Vertex {
    pub const fn get_binding_desc() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription {
            binding: 0,
            stride: mem::size_of::<Vertex>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        }
    }

    pub fn get_attribute_desc() -> Vec<vk::VertexInputAttributeDescription> {
        vec![
            vk::VertexInputAttributeDescription { location: 0, binding: 0, format: vk::Format::R32G32B32A32_SFLOAT, offset: offset_of!(Self, pos) as u32 },
            vk::VertexInputAttributeDescription { location: 1, binding: 0, format: vk::Format::R32G32B32A32_SFLOAT, offset: offset_of!(Self, color) as u32 },
            vk::VertexInputAttributeDescription { location: 2, binding: 0, format: vk::Format::R32G32_SFLOAT,       offset: offset_of!(Self, tex_coord) as u32 },
        ]
    }

    pub const fn structure_size() -> vk::DeviceSize {
        mem::size_of::<Self>() as vk::DeviceSize
    }
}