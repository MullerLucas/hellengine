use std::collections::HashMap;
use std::fs;
use std::path::Path;

use hell_core::config;
use hell_error::{HellResult, HellError, HellErrorKind, HellErrorContent};

use crate::ResourceManager;



// ----------------------------------------------------------------------------
// file data
// ----------------------------------------------------------------------------

#[derive(Debug, serde::Deserialize)]
pub struct MaterialFileWrapper {
    material: MaterialFile,
}

// ----------------------------------------------

#[derive(Debug, serde::Deserialize)]
pub struct MaterialFile {
    textures: HashMap<String, MaterialFileTextureData>,
}

// ----------------------------------------------

#[derive(Debug, serde::Deserialize)]
pub struct MaterialFileTextureData {
    pub path: String,
}




// ----------------------------------------------------------------------------
//  resource data
// ----------------------------------------------------------------------------
pub struct MaterialResource {
    textures: HashMap<String,  MaterialResourceTextureData>,
}

impl From<MaterialFile> for MaterialResource {
    fn from(f: MaterialFile) -> Self {
        let textures = f.textures.into_iter()
            .map(|(k, v)| { (k, MaterialResourceTextureData::from(v)) })
            .collect();

        Self {
            textures
        }
    }
}


impl MaterialResource {
    pub const MAIN_TEX: &'static str = "main_tex";
}

impl MaterialResource {
    pub fn load_from_disk(path: &str) -> HellResult<MaterialResource> {
        let path = Path::new(path);
        let raw = fs::read_to_string(path)?;
        let mat_file: MaterialFileWrapper = serde_yaml::from_str(&raw)?;

        Ok(MaterialResource::from(mat_file.material))
    }

    // TODO: move to resource-manager (maybe)
    pub fn load_texture(&mut self, resources: &mut ResourceManager) -> HellResult<()> {
        for (_, tex) in self.textures.iter_mut() {
            let idx = resources.load_image(tex.path.clone(), config::IMG_FLIP_V, config::IMG_FLIP_H)?;
            tex.idx = idx;
        }

        Ok(())
    }

    pub fn set_texture_idx(&mut self, key: &str, idx: usize) -> HellResult<()> {
        if let Some(tex) = self.textures.get_mut(key) {
            tex.idx = idx;
            Ok(())
        } else {
            Err(HellError::new(
                HellErrorKind::ResourceError,
                HellErrorContent::Message(format!("failed to set index for material texture '{}'", key))
            ))
        }
    }


    pub fn texture_at(&self, key: &str) -> Option<&MaterialResourceTextureData> {
        self.textures.get(key)
    }
}

// ------------------------------------

#[derive(Debug)]
pub struct MaterialResourceTextureData {
    pub path: String,
    pub idx: usize,
}

impl From<MaterialFileTextureData> for MaterialResourceTextureData {
    fn from(d: MaterialFileTextureData) -> Self {
        Self {
            path: d.path,
            idx: 0,
        }
    }
}

// ------------------------------------


