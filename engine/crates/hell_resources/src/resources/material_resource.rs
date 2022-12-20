use std::collections::HashMap;
use std::fs;
use std::path::Path;

use hell_error::HellResult;

use super::TextureInfo;





// ----------------------------------------------------------------------------
// file data
// ----------------------------------------------------------------------------

#[derive(Debug, serde::Deserialize)]
pub struct MaterialFile {
    material: MaterialInfo,
}

// ----------------------------------------------

#[derive(Debug, serde::Deserialize)]
pub struct MaterialInfo {
    name: String,
    shader: String,
    textures: HashMap<String, MaterialTextureInfo>,
}

// ----------------------------------------------

// #[derive(Debug, serde::Deserialize)]
// pub struct MaterialShaderInfo {
//     pub path: String,
// }

#[derive(Debug, serde::Deserialize)]
pub struct MaterialTextureInfo {
    pub path: String,
}




// ----------------------------------------------------------------------------
//  resource data
// ----------------------------------------------------------------------------

#[derive(Debug)]
pub struct MaterialResource {
    pub name: String,
    pub id: u64,
    pub id_internal: Option<u64>,
    pub shader: String,
    pub textures: HashMap<String, TextureInfo>,
}

impl From<MaterialInfo> for MaterialResource {
    fn from(info: MaterialInfo) -> Self {
        let textures = info.textures.into_iter()
            .map(|(k, v)| { (k.clone(), TextureInfo::new(k, v.path)) })
            .collect();

        // TODO:
        Self {
            id: 0,
            name: info.name,
            id_internal: None,
            shader: info.shader,
            textures,
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
        let file: MaterialFile = serde_yaml::from_str(&raw)?;

        Ok(MaterialResource::from(file.material))
    }
}

// ------------------------------------

