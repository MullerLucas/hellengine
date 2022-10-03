use std::collections::HashMap;
use std::fs;
use std::path::Path;

use hell_common::{HellError, HellResult};

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
pub struct MaterialResourceTextureData {
    path: String,
}

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
pub struct MaterialResource {
    textures: HashMap<String,  MaterialResourceTextureData>,
}


#[derive(Debug, serde::Deserialize)]
pub struct MaterialFile {
    material: MaterialResource,
}

impl MaterialResource {
    pub fn load_from_disk(path: &str) -> HellResult<MaterialResource> {
        let path = Path::new(path);
        let raw = fs::read_to_string(path).map_err(|e| HellError::from(format!("{}", e)))?;
        let mat_file: MaterialFile = serde_yaml::from_str(&raw).map_err(|e| HellError::from(format!("{}", e)))?;

        Ok(mat_file.material)
    }
}
