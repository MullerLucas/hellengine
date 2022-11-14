use hell_error::{HellErrorKind, HellError, HellResult};

use crate::resources::{ImageResource, MaterialResource};




// ----------------------------------------------------------------------------
// resource group
// ----------------------------------------------------------------------------

pub struct ResourceGroup<T> {
    paths: Vec<String>,
    resources: Vec<T>,
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

    pub fn len(&self) -> usize {
        self.paths.len()
    }

    pub fn contains(&self, path: &str) -> bool {
        self.paths.iter().any(|p| p == path)
    }

    pub fn add(&mut self, path: String, res: T) -> HellResult<usize> {
        if self.contains(&path) {
            return Err(HellError::from_msg(HellErrorKind::ResourceError, "trying to duplicate resource".to_owned()));
        }

        self.paths.push(path);
        self.resources.push(res);

        Ok(self.len() - 1)
    }
}




// ----------------------------------------------------------------------------
// resource manager
// ----------------------------------------------------------------------------

pub struct ResourceManager {
    images: ResourceGroup<ImageResource>,
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
    pub fn load_image(&mut self, path: String, flipv: bool, fliph: bool) -> HellResult<usize> {
        let img = ImageResource::load_from_disk(&path, flipv, fliph)?;
        self.images.add(path, img)
    }

    pub fn get_all_images(&self) -> &[ImageResource] {
        &self.images.resources
    }
}

impl ResourceManager {
    pub fn load_material(&mut self, path: &str) -> HellResult<usize> {
        // return already existing material
        // --------------------------------
        if let Some(idx) = self.materials.index_of(path) {
            return Ok(idx);
        }

        // load new material
        // -----------------
        let mut mat = MaterialResource::load_from_disk(path)?;
        mat.load_texture(self)?;
        self.materials.add(path.to_owned(), mat)
    }

    pub fn material_at(&self, idx: usize) -> Option<&MaterialResource> {
        self.materials.get_at(idx)
    }
}
