use std::collections::HashMap;

use hell_error::HellResult;

use crate::vulkan::{shader::generic_vulkan_shader::GenericVulkanShader, RenderBackend};

use super::ResourceHandle;

#[derive(Default)]
pub struct ShaderManager {
    handles:  HashMap<String, ResourceHandle>,
    // TODO: abstract vulkan specific details
    shaders: Vec<GenericVulkanShader>,
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

    pub fn create_shader(&mut self, backend: &RenderBackend, key: &str, global_tex: ResourceHandle) -> HellResult<ResourceHandle> {
        if let Some(handle) = self.handle(key) {
            Ok(handle)
        } else {
            println!("create shader '{}'", key);
            let handle = ResourceHandle::new(self.shaders.len());
            self.handles.insert(key.to_string(), handle);
            let shader = backend.shader_create(global_tex)?;
            self.shaders.push(shader);
            Ok(handle)
        }
    }

    pub fn shader(&self, handle: ResourceHandle) -> &GenericVulkanShader {
        self.shaders.get(handle.idx).unwrap()
    }

    pub fn shader_mut(&mut self, handle: ResourceHandle) -> &mut GenericVulkanShader {
        self.shaders.get_mut(handle.idx).unwrap()
    }
}
