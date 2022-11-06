use std::path::Path;

use hell_error::HellResult;
use image::{DynamicImage, RgbaImage};




pub struct ImageResource {
    rgba_img: RgbaImage,
}

impl ImageResource {
    pub fn load_from_disk(path: &str, flipv: bool) -> HellResult<Self> {
        let dyn_img = {
            let i = image::open(Path::new(path))?;
            if flipv { i.flipv() }
            else     { i }
        };

        let rgba_img: RgbaImage = match dyn_img {
            DynamicImage::ImageRgba8(img) => { img },
            DynamicImage::ImageRgb8(img)  => { DynamicImage::ImageRgb8(img).into_rgba8() },
            DynamicImage::ImageLuma8(img) => { DynamicImage::ImageLuma8(img).into_rgba8() },
            _ => { panic!("invalid image format"); }
        };

        Ok(Self {
            rgba_img,
        })
    }

    pub fn get_img(&self) -> &RgbaImage{
        &self.rgba_img
    }
}

