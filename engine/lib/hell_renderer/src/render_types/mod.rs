pub mod glsl;
pub mod scope;
pub mod stage;

use ash::vk;



// ----------------------------------------------------------------------------
// Per-Frame
// ----------------------------------------------------------------------------

pub type PerFrame<T> = [T; config::FRAMES_IN_FLIGHT];

pub const INVALID_USIZE: usize = usize::MAX;
pub const INVALID_U64: u64 = u64::MAX;


// ----------------------------------------------------------------------------
// render data
// ----------------------------------------------------------------------------

#[derive(Default)]
pub struct RenderPackage {
    pub world: RenderData,
    pub ui: RenderData,
}


// -----------------------------------------------

use hell_common::transform::Transform;

use crate::{resources::ResourceHandle, config};

pub struct RenderDataChunk<'a> {
    pub mesh_idx: usize,
    pub transform: &'a Transform,
    pub material: ResourceHandle,
}

// -----------------------------------------------

#[derive(Debug, Default)]
pub struct RenderData {
    pub meshes: Vec<usize>,
    pub transforms: Vec<Transform>,
    pub materials: Vec<ResourceHandle>,
}

impl RenderData {
    pub fn len(&self) -> usize {
        self.meshes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn add_data(&mut self, mesh_idx: usize, material: ResourceHandle, trans: Transform) -> usize {
        self.meshes.push(mesh_idx);
        self.transforms.push(trans);
        self.materials.push(material);

        self.len()
    }

    pub fn data_at(&self, idx: usize) -> RenderDataChunk {
        RenderDataChunk {
            mesh_idx: self.meshes[idx],
            transform: &self.transforms[idx],
            material: self.materials[idx]
        }
    }
}

impl RenderData {
    pub fn iter(&self) -> RenderDataIter {
        self.into_iter()
    }
}

impl<'a> IntoIterator for &'a RenderData {
    type Item = RenderDataChunk<'a>;
    type IntoIter = RenderDataIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        RenderDataIter::new(self)
    }
}

pub struct RenderDataIter<'a> {
    idx: usize,
    render_data: &'a RenderData,
}

impl<'a> RenderDataIter<'a> {
    pub fn new(render_data: &'a RenderData) -> RenderDataIter<'a> {
        Self {
            idx: 0,
            render_data,
        }
    }
}

impl<'a> Iterator for RenderDataIter<'a> {
    type Item = RenderDataChunk<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.render_data.len() > self.idx {
            let result = Some(self.render_data.data_at(self.idx));
            self.idx += 1;
            result
        } else {
            None
        }
    }
}

// ----------------------------------------------------------------------------


#[allow(non_camel_case_types)]
#[derive(Default, Debug, Clone, Copy)]
pub enum NumberFormat {
    #[default] UNDEFINED,
    R32G32_SFLOAT,
    R32G32B32_SFLOAT,
    R32G32B32A32_SFLOAT,
}

impl NumberFormat {
    const fn size_of<T>(count: usize) -> usize {
        std::mem::size_of::<T>() * count
    }

    pub const fn to_vk_format(&self) -> vk::Format {
        match self {
            NumberFormat::R32G32_SFLOAT       => vk::Format::R32G32_SFLOAT,
            NumberFormat::R32G32B32_SFLOAT    => vk::Format::R32G32B32_SFLOAT,
            NumberFormat::R32G32B32A32_SFLOAT => vk::Format::R32G32B32A32_SFLOAT,
            _ => vk::Format::UNDEFINED,
        }
    }

    pub const fn size(&self) -> usize {
        match self {
            NumberFormat::R32G32_SFLOAT       => Self::size_of::<f32>(2),
            NumberFormat::R32G32B32_SFLOAT    => Self::size_of::<f32>(3),
            NumberFormat::R32G32B32A32_SFLOAT => Self::size_of::<f32>(4),
            _ => 0,
        }
    }
}

// ----------------------------------------------

#[derive(Debug, Clone, Copy)]
pub struct ValueRange<T> {
    pub offset: T,
    pub range: T,
}

impl<T> ValueRange<T> {
    pub const fn new(offset: T, range: T) -> Self {
        Self { offset, range }
    }
}

impl<T> Default for ValueRange<T>
where T: Default
{
    fn default() -> Self {
        Self {
            offset: T::default(),
            range: T::default(),
        }
    }
}

pub type MemRange = ValueRange<usize>;

// ----------------------------------------------------------------------------

