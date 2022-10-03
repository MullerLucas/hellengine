use std::path::Path;

use hell_common::prelude::*;
use image::{DynamicImage, RgbaImage};




pub struct ImageResource {
    rgba_img: RgbaImage,
}

impl ImageResource {
    pub fn load_from_disk(path: &str, flipv: bool) -> HellResult<Self> {
        let dyn_img = image::open(Path::new(path))?;

        if flipv {
            dyn_img.flipv();
        }

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

