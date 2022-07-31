use ash::vk;

use std::io::Read;
use std::path::Path;
use std::{fs, ffi};

pub struct VulkanShader {
    pub vert_module: VulkanShaderModule,
    pub frag_module: VulkanShaderModule,
    stage_create_infos: [vk::PipelineShaderStageCreateInfo; 2],
}

impl VulkanShader {
    pub fn new(device: &ash::Device, vert_path: &str, frag_path: &str) -> Self {
        let vert_module = VulkanShaderModule::new(device, vert_path);
        let frag_module = VulkanShaderModule::new(device, frag_path);

        let stage_create_infos = [
            vert_module.stage_create_info(vk::ShaderStageFlags::VERTEX),
            frag_module.stage_create_info(vk::ShaderStageFlags::FRAGMENT),
        ];

        Self {
            vert_module,
            frag_module,
            stage_create_infos,
        }
    }

    pub fn get_stage_create_infos(&self) -> &[vk::PipelineShaderStageCreateInfo] {
        &self.stage_create_infos
    }
}


pub struct VulkanShaderModule {
    pub module: vk::ShaderModule,
}

impl VulkanShaderModule {
    pub fn new(device: &ash::Device, code_path: &str) -> Self {
        let code = read_shader_code(Path::new(code_path));
        let module = create_shader_module(device, &code);

        Self { module }
    }

    // TODO: error handling
    pub fn stage_create_info(&self, stage: vk::ShaderStageFlags) -> vk::PipelineShaderStageCreateInfo {
        let entrypoint = ffi::CString::new("main").unwrap();

        vk::PipelineShaderStageCreateInfo::builder()
            .stage(stage)
            .name(&entrypoint)
            .module(self.module)
            .build()
    }
}





// TODO: error handling
fn read_shader_code(path: &Path) -> Vec<u8> {
    let file = fs::File::open(path).unwrap();
    file.bytes().flatten().collect()
}

// TODO: error handling
fn create_shader_module(device: &ash::Device, code: &[u8]) -> vk::ShaderModule {
    // TODO: check
    let code = unsafe { std::mem::transmute::<&[u8], &[u32]>(code) };
    let module_info = vk::ShaderModuleCreateInfo::builder()
        .code(code)
        .build();

    unsafe {
        device.create_shader_module(&module_info, None).unwrap()
    }
}
