use ash::vk;
use hell_core::error::{HellResult, ErrToHellErr};

use std::io::Read;
use std::path::Path;
use std::{fs, ffi};

use crate::vulkan::VulkanContextRef;



pub struct VulkanShader {
    pub vert_module: VulkanShaderModule,
    pub frag_module: VulkanShaderModule,
    stage_create_infos: [vk::PipelineShaderStageCreateInfo; 2],
}

impl VulkanShader {
    pub fn from_file(ctx: &VulkanContextRef, path: &str) -> HellResult<Self> {
        println!("create vulkan shader from path: '{}'", path);
        let vert_path = format!("{}.vert.spv", path);
        let frag_path = format!("{}.frag.spv", path);

        let vert_module = VulkanShaderModule::new(ctx, &vert_path)?;
        let frag_module = VulkanShaderModule::new(ctx, &frag_path)?;

        let stage_create_infos = [
            vert_module.stage_create_info(vk::ShaderStageFlags::VERTEX),
            frag_module.stage_create_info(vk::ShaderStageFlags::FRAGMENT),
        ];

        Ok(Self {
            vert_module,
            frag_module,
            stage_create_infos,
        })
    }

    pub fn get_stage_create_infos(&self) -> &[vk::PipelineShaderStageCreateInfo] {
        &self.stage_create_infos
    }
}


// ----------------------------------------------


pub struct VulkanShaderModule {
    ctx: VulkanContextRef,
    pub entrypoint: ffi::CString,
    pub module: vk::ShaderModule,
}

impl Drop for VulkanShaderModule {
    fn drop(&mut self) {
        unsafe {
            let device = &self.ctx.device.handle;
            device.destroy_shader_module(self.module, None);
        }
    }
}

impl VulkanShaderModule {
    pub fn new(ctx: &VulkanContextRef, code_path: &str) -> HellResult<Self> {
        let entrypoint = ffi::CString::new("main").to_render_hell_err()?;
        let code = read_shader_code(Path::new(code_path))?;
        let module = create_shader_module(&ctx.device.handle, &code)?;

        Ok(Self {
            ctx: ctx.clone(),
            entrypoint,
            module
        })
    }

    pub fn stage_create_info(&self, stage: vk::ShaderStageFlags) -> vk::PipelineShaderStageCreateInfo {
        vk::PipelineShaderStageCreateInfo::builder()
            .stage(stage)
            .name(&self.entrypoint)
            .module(self.module)
            .build()
    }
}

fn read_shader_code(path: &Path) -> HellResult<Vec<u8>> {
    let file = fs::File::open(path).to_render_hell_err()?;
    Ok(
        file.bytes()
            .filter_map(|b| b.ok())
            .collect()
    )
}

fn create_shader_module(device: &ash::Device, code: &[u8]) -> HellResult<vk::ShaderModule> {
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
        device.create_shader_module(&module_info, None).to_render_hell_err()
    }
}
