use std::{collections::HashMap, borrow::Borrow, path::Path, fs};
use std::fmt::Write;

use hell_core::collections::dyn_array::DynArray;
use hell_core::error::{HellResult, OptToHellErr};
use hell_renderer::render_types::scope::ShaderScopeType;
use hell_renderer::resources::config::{ShaderProgramConfig, ShaderProgramInfoConfig, ShaderProgramScopeConfig, ShaderProgramBufferConfig, ShaderProgramUboVarConfig, ShaderProgramSamplerConfig, ShaderProgramShaderConfig, ShaderProgramUniformUsage};
use pest::{self, Parser, iterators::{Pair, Pairs}};
use pest_derive::{self, Parser};



#[derive(Parser)]
#[grammar = "pest/test.pest"]
pub struct CrapParser;

pub fn run() -> HellResult<()> {
    let input = std::fs::read_to_string("pest/test.glsl").unwrap();
    let file = CrapParser::parse(Rule::file, &input).unwrap()
        .next().unwrap()
        .into_inner();

    let mut result = CrapFile::new();

    for pair in file {
        match pair.as_rule() {
            Rule::info_decl   => {
                result.info = Some(CrapInfoDef::new(pair.into_inner()));
            },
            Rule::scope_decl  => {
                let scope = CrapScopeDef::new(pair.into_inner());
                result.scopes.push(scope);
            },
            Rule::shader_decl => {
                let shader = CrapShaderDef::new(pair.into_inner());
                result.shaders.push(shader);
            },
            Rule::EOI => { }
            _ => {
                unreachable!();
            }
        }
    }

    let mut config: ShaderProgramConfig = result.borrow().into();
    config.update_sets_and_bindings();

    let shader_file = config.info.generate_path();
    let out_dir = std::path::Path::new("./generated");
    std::fs::write(
        out_dir.join(format!("{}_DEF.json", shader_file)),
        serde_json::to_string_pretty(&config).unwrap()
    ).unwrap();
    std::fs::write(
        out_dir.join(format!("{}_DEF.yaml", shader_file)),
        serde_yaml::to_string(&config).unwrap()
    ).unwrap();

    result.shaders.as_slice().iter().for_each(|s| s.write_files(out_dir, shader_file.clone(), &config).unwrap());

    Ok(())
}

// ----------------------------------------------------------------------------

#[derive(Debug)]
pub struct CrapFile<'a> {
    pub info: Option<CrapInfoDef<'a>>,
    pub scopes: DynArray<CrapScopeDef<'a>, {CrapFile::MAX_SCOPES}>,
    pub shaders: DynArray<CrapShaderDef<'a>, {CrapFile::MAX_SHADERS}>,
}

impl<'a> CrapFile<'a> {
    pub const MAX_SCOPES: usize = 10;
    pub const MAX_SHADERS: usize = 10;

    pub fn new() -> Self {
        Self {
            info: None,
            scopes: Default::default(),
            shaders: Default::default(),
        }
    }
}

impl Into<ShaderProgramConfig> for &CrapFile<'_> {
    fn into(self) -> ShaderProgramConfig {
        ShaderProgramConfig {
            info: self.info.as_ref().unwrap().into(),
            scopes: self.scopes.as_slice().iter().map(|s| s.into()).collect(),
            shaders: self.shaders.as_slice().iter().map(|s| s.into()).collect(),
        }
    }
}

// ----------------------------------------------------------------------------

#[derive(Debug)]
pub struct CrapInfoDef<'a> {
    pub fields: HashMap<&'a str, CrapInfoVarDef<'a>>,
}

impl<'a> CrapInfoDef<'a> {
    pub fn new(mut pairs: Pairs<'a, Rule>) -> Self {
        let mut info_block = pairs.next().unwrap().into_inner();
        let mut fields = HashMap::new();

        while let Some(Rule::info_var) = info_block.peek().and_then(|b| Some(b.as_rule())) {
            let field = CrapInfoVarDef::new(info_block.next().unwrap().into_inner());
            fields.insert(field.ident, field);
        }

        Self { fields }
    }
}

impl Into<ShaderProgramInfoConfig> for &CrapInfoDef<'_> {
    fn into(self) -> ShaderProgramInfoConfig {
        ShaderProgramInfoConfig::from_raw(
            self.fields["version"].value,
            self.fields["name"].value,
            self.fields["pass"].value,
        )
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug)]
pub struct CrapInfoVarDef<'a> {
    pub ident: &'a str,
    pub value: &'a str,
}

impl<'a> CrapInfoVarDef<'a> {
    pub fn new(mut pairs: Pairs<'a, Rule>) -> Self {
        let ident = pairs.next().unwrap().as_str();
        let val = pairs.next().unwrap().as_str();

        Self {
            ident,
            value: val,
        }
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct CrapScopeDef<'a> {
    pub name: &'a str,
    pub buffers: DynArray<CrapUniformBufferDef<'a>, {CrapScopeDef::MAX_BUFFERS}>,
    pub samplers: DynArray<CrapUniformSamplerDef<'a>, {CrapScopeDef::MAX_SAMPLERS}>,
}

impl<'a> CrapScopeDef<'a> {
    pub const MAX_BUFFERS: usize = 10;
    pub const MAX_SAMPLERS: usize = 10;

    pub fn new(mut pairs: Pairs<'a, Rule>) -> Self {
        let name = pairs.next().unwrap().as_str();
        let scope_block = pairs.next().unwrap().into_inner();

        let mut buffers = DynArray::default();
        let mut samplers = DynArray::default();

        for pair in scope_block {
            match pair.as_rule() {
                Rule::uniform_buffer => {
                    buffers.push(CrapUniformBufferDef::new(pair.into_inner()));
                }
                Rule::uniform_sampler => {
                    samplers.push(CrapUniformSamplerDef::new(pair.into_inner()));
                },
                _ => unreachable!()
            }
        }

        Self {
            name,
            buffers,
            samplers,
        }
    }
}

impl Into<ShaderProgramScopeConfig> for &CrapScopeDef<'_> {
    fn into(self) -> ShaderProgramScopeConfig {
        ShaderProgramScopeConfig::from_raw(
            self.name,
            self.buffers.as_slice().iter().map(|b| b.into()).collect(),
            self.samplers.as_slice().iter().map(|s| s.into()).collect(),
        ).unwrap()
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct CrapUniformBufferDef<'a> {
    pub var_ubos: DynArray<CrapVarUboDef<'a>, {CrapUniformBufferDef::MAX_VARS}>,
    pub ident: &'a str,
}

impl<'a> CrapUniformBufferDef<'a> {
    pub const MAX_VARS: usize = 20;

    pub fn new(mut pairs: Pairs<'a, Rule>) -> Self {
        let mut var_ubos = DynArray::from_fn(|_| CrapVarUboDef::default());

        while let Rule::var_ubo = pairs.peek().unwrap().as_rule() {
            var_ubos.push(CrapVarUboDef::new(pairs.next().unwrap().into_inner()));
        }

        let ident = pairs.next().unwrap().as_str();

        Self {
            ident,
            var_ubos,
        }
    }
}

impl Into<ShaderProgramBufferConfig> for &CrapUniformBufferDef<'_> {
    fn into(self) -> ShaderProgramBufferConfig {
        ShaderProgramBufferConfig::from_raw(
            self.ident,
            self.var_ubos.as_slice().iter().map(|v| v.into()).collect(),
        ).unwrap()
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct CrapVarUboDef<'a> {
    pub type_ubo: &'a str,
    pub ident: &'a str
}

impl<'a> CrapVarUboDef<'a> {
    pub fn new(mut pairs: Pairs<'a, Rule>) -> Self {
        let type_ubo = pairs.next().unwrap().as_str();
        let ident = pairs.next().unwrap().as_str();

        Self {
            type_ubo,
            ident,
        }
    }
}

impl Into<ShaderProgramUboVarConfig> for &CrapVarUboDef<'_> {
    fn into(self) -> ShaderProgramUboVarConfig {
        ShaderProgramUboVarConfig::from_raw(self.type_ubo, self.ident).unwrap()
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct CrapUniformSamplerDef<'a> {
    pub type_sampler: &'a str,
    pub ident: &'a str,
}

impl<'a> CrapUniformSamplerDef<'a> {
    pub fn new(mut pairs: Pairs<'a, Rule>) -> Self {
        let type_sampler = pairs.next().unwrap().as_str();
        let ident = pairs.next().unwrap().as_str();

        Self {
            ident,
            type_sampler,
        }
    }
}

impl Into<ShaderProgramSamplerConfig> for &CrapUniformSamplerDef<'_> {
    fn into(self) -> ShaderProgramSamplerConfig {
        ShaderProgramSamplerConfig::from_raw(self.type_sampler, self.ident).unwrap()
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct CrapShaderDef<'a> {
    pub ident: &'a str,
    pub uniform_usages: DynArray<CrapUniformUsage<'a>, {CrapShaderDef::MAX_UNIFORM_USAGES}>,
    pub raw_code: CrapRawCode<'a>,
}

impl<'a> CrapShaderDef<'a> {
    pub const MAX_UNIFORM_USAGES: usize = 100;

    pub fn new(mut pairs: Pairs<'a, Rule>) -> Self {
        let name = pairs.next().unwrap().as_str();
        let mut shader_block = pairs.next().unwrap().into_inner();

        let mut uniform_usages = DynArray::default();

        while let Rule::uniform_usage = shader_block.peek().unwrap().as_rule() {
            uniform_usages.push(CrapUniformUsage::new(shader_block.next().unwrap().into_inner()))
        }

        let raw_code = CrapRawCode::new(shader_block.next().unwrap());

        Self {
            ident: name,
            uniform_usages,
            raw_code,
        }
    }

    pub fn write_code_block(&self, buffer: &mut String) {
        buffer.push_str("// --- START: code ---\n");
        buffer.push_str(self.raw_code.code);
        buffer.push_str("\n// --- END: code ---");
    }

    pub fn write_files(&self, base_path: &Path, file_stem: String, config: &ShaderProgramConfig) -> HellResult<()> {
        let file_name = format!("{}_{}.glsl", file_stem, self.ident);
        let file_path = base_path.join(file_name);

        let mut code = String::from("\n");
        code.insert_str(0, "\t");

        for usage in self.uniform_usages.as_slice() {
            let target_scope = ShaderScopeType::try_from(usage.scope_type).unwrap();
            let scope = config.scope_ref(target_scope).ok_or_render_herr("failed to get scope while writing shader file")?;

            if let Some(buffer) = scope.buffer(usage.ident) {
                buffer.format(scope.scope_type, &mut code)?;
                writeln!(code, "")?;
            } else if let Some(sampler) = scope.sampler(usage.ident) {
                writeln!(code, "{}", sampler)?;
            } else {
                write!(code, "\n// ERROR: failed to write uniform '{}::{}'\n\n", usage.scope_type, usage.ident)?;
            }
        }

        self.write_code_block(&mut code);

        fs::create_dir_all(base_path)?;
        fs::write(file_path, code)?;

        Ok(())
    }
}

impl Into<ShaderProgramShaderConfig> for &CrapShaderDef<'_> {
    fn into(self) -> ShaderProgramShaderConfig {
        ShaderProgramShaderConfig::from_raw(
            self.ident,
            self.uniform_usages.as_slice().iter().map(|u| u.into()).collect(),
        ).unwrap()
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct CrapUniformUsage<'a> {
    pub scope_type: &'a str,
    pub ident: &'a str,
}

impl<'a> CrapUniformUsage<'a> {
    pub fn new(mut pairs: Pairs<'a, Rule>) -> Self {
        let scope_ident = pairs.next().unwrap().as_str();
        let field_ident = pairs.next().unwrap().as_str();

        Self {
            scope_type: scope_ident,
            ident: field_ident,
        }
    }
}

impl Into<ShaderProgramUniformUsage> for &CrapUniformUsage<'_> {
    fn into(self) -> ShaderProgramUniformUsage {
        ShaderProgramUniformUsage::from_raw(self.scope_type, self.ident).unwrap()
    }
}

// -----------------------------------------------------------------------------

#[derive(Debug, Default)]
pub struct CrapRawCode<'a> {
    pub code: &'a str,
}

impl<'a> CrapRawCode<'a> {
    pub fn new(pair: Pair<'a, Rule>) -> Self {
        let code = pair.as_str();

        Self {
            code,
        }
    }
}


// -----------------------------------------------------------------------------
