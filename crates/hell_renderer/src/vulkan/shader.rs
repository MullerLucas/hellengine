use ash::vk;

use std::io::Read;
use std::path::Path;
use std::{fs, ffi};



pub struct Shader {
    pub vert_module: VulkanShaderModule,
    pub frag_module: VulkanShaderModule,
    stage_create_infos: [vk::PipelineShaderStageCreateInfo; 2],
}

impl Shader {
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

    pub fn drop_manual(&self, device: &ash::Device) {
        println!("> dropping Shader...");

        unsafe {
            device.destroy_shader_module(self.vert_module.module, None);
            device.destroy_shader_module(self.frag_module.module, None);
        }
    }
}



pub struct VulkanShaderModule {
    pub entrypoint: ffi::CString,
    pub module: vk::ShaderModule,
}

impl VulkanShaderModule {
    pub fn new(device: &ash::Device, code_path: &str) -> Self {
        let entrypoint = ffi::CString::new("main").unwrap();
        let code = read_shader_code(Path::new(code_path));
        let module = create_shader_module(device, &code);

        Self { entrypoint, module }
    }

    // TODO: error handling
    pub fn stage_create_info(&self, stage: vk::ShaderStageFlags) -> vk::PipelineShaderStageCreateInfo {
        vk::PipelineShaderStageCreateInfo::builder()
            .stage(stage)
            .name(&self.entrypoint)
            .module(self.module)
            .build()
    }
}





// TODO: error handling
fn read_shader_code(path: &Path) -> Vec<u8> {
    let file = fs::File::open(path).unwrap();
    // file.bytes().flatten().collect()
    file.bytes().filter_map(|b| b.ok()).collect()
}

// TODO: error handling
fn create_shader_module(device: &ash::Device, code: &[u8]) -> vk::ShaderModule {
    // TODO: check
    // let code = unsafe { std::mem::transmute::<&[u8], &[u32]>(code) };
    // let module_info = vk::ShaderModuleCreateInfo::builder()
    //     .code(code)
    //     .build();

    let module_info = vk::ShaderModuleCreateInfo {
        s_type: vk::StructureType::SHADER_MODULE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::ShaderModuleCreateFlags::empty(),
        code_size: code.len(),
        p_code: code.as_ptr() as *const u32,
    };

    unsafe {
        device.create_shader_module(&module_info, None).unwrap()
    }
}
