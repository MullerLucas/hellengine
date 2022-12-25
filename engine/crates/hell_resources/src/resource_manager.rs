use std::collections::HashSet;

use hell_error::{HellErrorKind, HellError, HellResult};
use crate::resource_config;

use crate::resources::{TextureResource, MaterialResource};




// ----------------------------------------------------------------------------
// resource group
// ----------------------------------------------------------------------------

pub struct ResourceGroup<T> {
    pub paths: Vec<String>,
    pub resources: Vec<T>,
}

impl<T> Default for ResourceGroup<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> ResourceGroup<T> {
    pub fn new() -> Self {
        Self {
            paths: Vec::new(),
            resources: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn get_all(&self) -> &[T] {
        &self.resources
    }

    pub fn get_at(&self, idx: usize) -> Option<&T> {
        self.resources.get(idx)
    }

    pub fn index_of(&self, path: &str) -> Option<usize> {
        self.paths.iter()
            .position(|p| p == path)
    }

    #[allow(dead_code)]
    pub fn get(&self, path: &str) -> Option<&T> {
        self.index_of(path)
            .and_then(|i| self.get_at(i))
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }


    pub fn len(&self) -> usize {
        self.paths.len()
    }

    pub fn contains(&self, path: &str) -> bool {
        self.paths.iter().any(|p| p == path)
    }

    pub fn add(&mut self, path: String, res: T) -> HellResult<ResourceHandle> {
        if self.contains(&path) {
            return Err(HellError::from_msg(HellErrorKind::ResourceError, "trying to duplicate resource".to_owned()));
        }

        self.paths.push(path);
        self.resources.push(res);

        Ok(ResourceHandle::from(self.len() - 1))
    }
}




// ----------------------------------------------------------------------------
// Resource-Handle
// ----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResourceHandle {
    pub id: usize,
}

impl From<usize> for ResourceHandle {
    fn from(value: usize) -> Self {
        Self { id: value }
    }
}

impl ResourceHandle {
    pub const MAX: Self = Self { id: usize::MAX };
}


// ----------------------------------------------------------------------------
// resource manager
// ----------------------------------------------------------------------------

pub struct ResourceManager {
    images: ResourceGroup<TextureResource>,
    materials: ResourceGroup<MaterialResource>,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            images: ResourceGroup::new(),
            materials: ResourceGroup::new(),
        }
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ResourceManager {
    pub fn load_image(&mut self, path: String, flipv: bool, fliph: bool) -> HellResult<ResourceHandle> {
        let img = TextureResource::load_from_disk(&path, flipv, fliph)?;
        self.images.add(path, img)
    }

    pub fn get_all_images(&self) -> &[TextureResource] {
        &self.images.resources
    }
}


impl ResourceManager {
    pub fn load_material(&mut self, path: &str) -> HellResult<ResourceHandle> {
        // return already existing material
        // --------------------------------
        if let Some(idx) = self.materials.index_of(path) {
            return Ok(ResourceHandle::from(idx));
        }

        // load new material
        // ----------------
        let mat = MaterialResource::load_from_disk(path)?;
        // TODO: load texture
        // mat.load_texture(self)?;
        self.materials.add(path.to_owned(), mat)
    }

    pub fn load_used_textures(&mut self) -> HellResult<()> {
        let imgs_to_load: Vec<String> = self.materials.resources
            .iter()
            .flat_map(|m| {
                let t: Vec<String> = m.textures.values().map(|v| v.path.clone()).collect();
                t
            })
            .collect();

        for path in imgs_to_load {
            self.load_image(path, resource_config::IMG_FLIP_V, resource_config::IMG_FLIP_H)?;
        }

        Ok(())
    }

    pub fn unique_shader_keys(&self) -> HashSet<String> {
        self.materials.resources.iter()
            .map(|m| m.shader.clone())
            .collect()
    }

    pub fn material_at(&self, idx: usize) -> Option<&MaterialResource> {
        self.materials.get_at(idx)
    }
}
