use hell_common::{HellResult, HellError};

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
    pub fn get_resources(&self) -> &[T] {
        &self.resources
    }

    pub fn len(&self) -> usize {
        self.paths.len()
    }

    pub fn contains(&self, path: &str) -> bool {
        self.paths.iter().any(|p| p == path)
    }

    pub fn add(&mut self, path: String, res: T) -> HellResult<usize> {
        if self.contains(&path) {
            return Err(HellError::from("trying to duplicate resource".to_owned()));
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
    pub fn load_image(&mut self, path: &str, flipv: bool) -> HellResult<usize> {
        let img = ImageResource::load_from_disk(path, flipv)?;
        self.images.add(path.to_owned(), img)
    }

    pub fn get_all_images(&self) -> &[ImageResource] {
        &self.images.resources
    }
}

impl ResourceManager {
    pub fn load_material(&mut self, path: &str) -> HellResult<usize> {
        let mat = MaterialResource::load_from_disk(path)?;

        self.materials.add(path.to_owned(), mat)
    }
}
