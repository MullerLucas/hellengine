use std::{path::Path, collections::HashMap};

use hell_core::error::{HellResult, HellErrorHelper};
use image::{RgbaImage, DynamicImage};

use crate::vulkan::{RenderTexture, RenderBackend};

use super::ResourceHandle;




pub struct TextureManager {
    handles:  HashMap<String, ResourceHandle>,
    images:   Vec<Option<RgbaImage>>,
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

    pub fn acquire_textuer(&mut self, backend: &RenderBackend, key: String, path: Option<String>, flipv: bool, fliph: bool) -> HellResult<ResourceHandle> {
        if let Some(handle) = self.handle(&key) {
            return Ok(handle);
        }

        let (img, internal) = if let Some(path) = path {
            let img = Self::load_img(&path, flipv, fliph)?;
            let data = img.as_raw().as_slice();
            let internal = backend.texture_create(data, img.width() as usize, img.height() as usize)?;
            (Some(img), internal)
        } else {
            let internal = backend.texture_create_default()?;
            (None, internal)
        };

        let handle = ResourceHandle::new(self.textures.len());
        self.handles.insert(key, handle);
        self.images.push(img);
        self.textures.push(internal);

        Ok(handle)
    }

    pub fn handle(&self, path: &str) -> Option<ResourceHandle> {
        self.handles.get(path).copied()
    }

    pub fn textures(&self) -> &[RenderTexture] {
        &self.textures
    }

    pub fn texture(&self, handle: ResourceHandle) -> Option<&RenderTexture> {
        self.textures.get(handle.idx)
    }

    pub fn texture_res(&self, handle: ResourceHandle) -> HellResult<&RenderTexture> {
        self.textures.get(handle.idx).ok_or_else(|| HellErrorHelper::render_msg_err("failed to get texture"))
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
