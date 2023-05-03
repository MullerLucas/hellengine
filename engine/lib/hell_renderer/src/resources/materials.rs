use std::{collections::HashMap, path::Path, fs};

use hell_core::error::HellResult;

use crate::vulkan::RenderBackend;

use super::{ResourceHandle, TextureManager};



// ----------------------------------------------------------------------------

#[derive(Debug, serde::Deserialize)]
pub struct MaterialFile {
    pub material: MaterialInfo,
}

#[derive(Debug, serde::Deserialize)]
pub struct MaterialInfo {
    pub name: String,
    pub shader: String,
    pub textures: HashMap<String, MaterialTextureInfo>,
}

#[derive(Debug, serde::Deserialize)]
pub struct MaterialTextureInfo {
    pub path: String,
}

// ----------------------------------------------------------------------------

#[derive(Default)]
pub struct MaterialManager {
    handles: HashMap<String, ResourceHandle>,
    shader: Vec<String>,
    textures: Vec<HashMap<String, ResourceHandle>>,
}

impl MaterialManager {
    pub const MAIN_TEX: &'static str = "main_tex";
}

impl MaterialManager {
    pub fn new() -> Self {
        Self {
            handles: Default::default(),
            shader: Default::default(),
            textures: Default::default(),
        }
    }

    pub fn len(&self) -> usize {
        self.shader.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn handle(&self, path: &str) -> Option<ResourceHandle> {
        self.handles.get(path).copied()
    }

    pub fn acquire(&mut self, backend: &RenderBackend, tex_man: &mut TextureManager, path: String, info: MaterialInfo) -> HellResult<ResourceHandle> {
        if let Some(handle) = self.handle(&path) {
            return Ok(handle);
        }

        let handle = ResourceHandle::new(self.len());

        let textures: HellResult<HashMap<_, _>> = info.textures.into_iter()
            .map(|(k, v)| {
                let handle = tex_man.acquire_textuer(backend, v.path.clone(), Some(v.path), false, false)?;
                Ok((k, handle))
            })
            .collect();
        let textures = textures?;

        self.handles.insert(path, handle);
        self.shader.push(info.shader);
        self.textures.push(textures);

        Ok(handle)
    }

    pub fn acquire_from_file(&mut self, backend: &RenderBackend, tex_man: &mut TextureManager, path: String) -> HellResult<ResourceHandle> {
        let file = Self::load_file(&path)?;
        self.acquire(backend, tex_man, path, file.material)
    }
}

impl MaterialManager {
    fn load_file(path: &str) -> HellResult<MaterialFile> {
        let path = Path::new(path);
        let raw = fs::read_to_string(path)?;
        let file: MaterialFile = serde_yaml::from_str(&raw)?;
        Ok(file)
    }
}
