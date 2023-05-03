pub mod config;


use std::collections::HashMap;

use hell_core::error::{HellResult, OptToHellErr};

use crate::vulkan::{shader_program::ShaderProgram, RenderBackend};

use super::ResourceHandle;

#[derive(Default)]
pub struct ShaderManager {
    handles:  HashMap<String, ResourceHandle>,
    // TODO: abstract vulkan specific details
    shaders: Vec<ShaderProgram>,
}

impl ShaderManager {
    pub fn new() -> Self {
        Self {
            handles: HashMap::new(),
            shaders: Vec::new(),
        }
    }

    pub fn handle(&self, key: &str) -> Option<ResourceHandle> {
        self.handles.get(key).copied()
    }

    pub fn handle_res(&self, key: &str) -> HellResult<ResourceHandle> {
        self.handles.get(key).copied().ok_or_render_herr("failed to get shader handle")
    }

    pub fn create_shader(&mut self, backend: &RenderBackend, key: &str, global_tex: ResourceHandle, is_sprite_shader: bool) -> HellResult<ResourceHandle> {
        if let Some(handle) = self.handle(key) {
            Ok(handle)
        } else {
            println!("create shader '{}'", key);
            let handle = ResourceHandle::new(self.shaders.len());
            self.handles.insert(key.to_string(), handle);
            let shader = if is_sprite_shader { backend.create_sprite_shader(global_tex)? } else { backend.create_test_shader(global_tex)? };
            self.shaders.push(shader);
            Ok(handle)
        }
    }

    pub fn shader(&self, handle: ResourceHandle) -> &ShaderProgram {
        self.shaders.get(handle.idx).unwrap()
    }

    pub fn shader_mut(&mut self, handle: ResourceHandle) -> &mut ShaderProgram {
        self.shaders.get_mut(handle.idx).unwrap()
    }
}
