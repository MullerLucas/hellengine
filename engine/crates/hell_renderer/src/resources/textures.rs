use std::{path::Path, collections::HashMap};

use hell_error::HellResult;
use image::{RgbaImage, DynamicImage};

use crate::vulkan::{RenderBackend, RenderTexture, primitives::VulkanTexture};

use super::ResourceHandle;




pub struct TextureManager {
    handles:  HashMap<String, ResourceHandle>,
    images:   Vec<RgbaImage>,
    textures: Vec<RenderTexture>,
}

impl Default for TextureManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TextureManager {
    pub fn new() -> Self {
        Self {
            handles: HashMap::new(),
            images:  Vec::new(),
            textures: Vec::new(),
        }
    }

    pub fn acquire_textuer(&mut self, backend: &RenderBackend, path: String, flipv: bool, fliph: bool) -> HellResult<ResourceHandle> {
        if let Some(handle) = self.handle(&path) {
            return Ok(handle);
        }

        let img = Self::load_img(&path, flipv, fliph)?;
        let data = img.as_raw().as_slice();
        let internal = backend.create_texture(data, img.width() as usize, img.height() as usize)?;

        let handle = ResourceHandle::new(self.images.len());
        self.handles.insert(path, handle);
        self.images.push(img);
        self.textures.push(internal);

        Ok(handle)
    }

    pub fn handle(&self, path: &str) -> Option<ResourceHandle> {
        self.handles.get(path).copied()
    }

    pub fn textures(&self) -> &[VulkanTexture] {
        &self.textures
    }

    pub fn texture(&self, handle: ResourceHandle) -> Option<&RenderTexture> {
        self.textures.get(handle.id)
    }
}

impl TextureManager {
    fn load_img(path: &str, flipv: bool, fliph: bool) -> HellResult<RgbaImage> {
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

        Ok(rgba_img)
    }

}
