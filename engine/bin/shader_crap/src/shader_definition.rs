use std::collections::HashMap;

use hell_collections::DynArray;
use hell_error::HellResult;

use crate::{GlslType, ShaderScopeType};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ShaderProgramConfig {
    pub info: ShaderProgramInfoConfig,
    pub scopes: HashMap<String, ShaderProgramScopeConfig>,
    pub shaders: HashMap<String, ShaderProgramShaderConfig>
}

// ----------------------------------------------------------------------------

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ShaderProgramInfoConfig {
    pub version: String,
    pub name: String
}

impl ShaderProgramInfoConfig {
    pub fn from_raw(version: &str, name: &str) -> Self {
        Self {
            version: version.to_lowercase(),
            name: name.to_lowercase().replace("\"", ""),
        }
    }

    pub fn generate_path(&self) -> String {
        self.name.replace("/", "_")
    }
}

// ----------------------------------------------------------------------------

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ShaderProgramScopeConfig {
    pub name: String,
    pub buffers: Vec<ShaderProgramBufferConfig>,
    pub samplers: Vec<ShaderProgramSamplerConfig>,
}

impl ShaderProgramScopeConfig {
    pub fn from_raw(name: &str, buffers: Vec<ShaderProgramBufferConfig>, samplers: Vec<ShaderProgramSamplerConfig>) -> Self {
        Self {
            name: name.to_lowercase(),
            buffers,
            samplers,
        }
    }
}

// ----------------------------------------------------------------------------

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ShaderProgramShaderConfig {
    pub name: String,
    pub uniform_usages: Vec<ShaderProgramUniformUsage>,
}

impl ShaderProgramShaderConfig {
    pub fn from_raw(name: &str, uniform_usages: Vec<ShaderProgramUniformUsage>) -> Self {
        Self {
            name: name.to_lowercase(),
            uniform_usages,
        }
    }
}

// ----------------------------------------------------------------------------

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ShaderProgramBufferConfig {
    pub ident: String,
    pub var_ubos: Vec<ShaderProgramUboVarConfig>,
}

impl ShaderProgramBufferConfig {
    pub fn from_raw(ident: &str, var_ubos: Vec<ShaderProgramUboVarConfig>) -> HellResult<Self> {
        Ok(Self {
            ident: ident.to_lowercase(),
            var_ubos,
        })
    }
}

// ----------------------------------------------------------------------------

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ShaderProgramUboVarConfig {
    pub type_ubo: GlslType,
    pub ident: String
}

impl ShaderProgramUboVarConfig {
    pub fn from_raw(type_ubo: &str, ident: &str) -> HellResult<Self> {
        Ok(Self {
            type_ubo: GlslType::try_from(type_ubo)?,
            ident: ident.to_lowercase(),
        })
    }
}


// ----------------------------------------------------------------------------

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ShaderProgramSamplerConfig {
    pub type_sampler: GlslType,
    pub ident: String,
}

impl ShaderProgramSamplerConfig {
    pub fn from_raw(type_sampler: &str, ident: &str) -> HellResult<Self> {
        Ok(Self {
            type_sampler: GlslType::try_from(type_sampler)?,
            ident: ident.to_lowercase(),
        })
    }
}

// ----------------------------------------------------------------------------

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ShaderProgramUniformUsage {
    pub scope_type: ShaderScopeType,
    pub ident: String,
}

impl ShaderProgramUniformUsage {
    pub fn from_raw(scope_ident: &str, field_ident: &str) -> HellResult<Self> {
        Ok(Self {
            scope_type: ShaderScopeType::try_from(scope_ident)?,
            ident: field_ident.to_lowercase(),
        })
    }
}

// ----------------------------------------------------------------------------
