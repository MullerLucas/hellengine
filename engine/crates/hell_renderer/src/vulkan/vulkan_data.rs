
// ----------------------------------------------------------------------------
// render data chunk
// ----------------------------------------------------------------------------

use hell_common::transform::Transform;
use hell_resources::ResourceHandle;

pub struct RenderDataChunk<'a> {
    pub mesh_idx: usize,
    pub transform: &'a Transform,
    pub material: ResourceHandle,
}



// ----------------------------------------------------------------------------
// render data
// ----------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct RenderData {
    pub meshes: Vec<usize>,
    pub transforms: Vec<Transform>,
    pub materials: Vec<ResourceHandle>,
}

impl RenderData {
    pub fn data_count(&self) -> usize {
        self.meshes.len()
    }

    pub fn add_data(&mut self, mesh_idx: usize, material: ResourceHandle, trans: Transform) -> usize {
        self.meshes.push(mesh_idx);
        self.transforms.push(trans);
        self.materials.push(material);

        self.data_count()
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
        if self.render_data.data_count() > self.idx {
            let result = Some(self.render_data.data_at(self.idx));
            self.idx += 1;
            result
        } else {
            None
        }
    }
}
