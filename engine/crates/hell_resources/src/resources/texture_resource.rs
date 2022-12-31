use std::path::Path;

use hell_error::HellResult;
use image::{DynamicImage, RgbaImage};


// ----------------------------------------------------------------------------
// info
// ----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TextureInfo {
    pub id_internal: Option<u64>,
    pub name: String,
    pub path: String,
}

impl TextureInfo {
    pub fn new(name: String, path: String) -> Self {
        Self {
            id_internal: None,
            name,
            path,
        }
    }
}



// ----------------------------------------------------------------------------
// resource
// ----------------------------------------------------------------------------

#[allow(dead_code)]
pub struct TextureResource {
    id_internal: Option<u64>,
    rgba_img: RgbaImage,
}

impl TextureResource {
    pub fn load_from_disk(path: &str, flipv: bool, fliph: bool) -> HellResult<Self> {
        let dyn_img = {
            let i = image::open(Path::new(path))?;
            let tmp = if flipv { i.flipv() } else { i };
            if fliph { tmp.fliph() } else { tmp }
        };

        let rgba_img: RgbaImage = match dyn_img {
            DynamicImage::ImageRgba8(img) => { img },
            DynamicImage::ImageRgb8(img)  => { DynamicImage::ImageRgb8(img).into_rgba8() },
            DynamicImage::ImageLuma8(img) => { DynamicImage::ImageLuma8(img).into_rgba8() },
            _ => { panic!("invalid image format"); }
        };

        Ok(Self {
            id_internal: None,
            rgba_img,
        })
    }

    pub fn img(&self) -> &RgbaImage{
        &self.rgba_img
    }
}

