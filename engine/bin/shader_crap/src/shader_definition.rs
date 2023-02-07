use std::collections::HashMap;

use hell_collections::DynArray;
use hell_error::HellResult;

use crate::{GlslType, ShaderScopeType, ShaderType};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ShaderProgramConfig {
    pub info: ShaderProgramInfoConfig,
    pub scopes: Vec<ShaderProgramScopeConfig>,
    pub shaders: Vec<ShaderProgramShaderConfig>
}

impl ShaderProgramConfig {
    pub fn scope_ref(&self, scope_type: ShaderScopeType) -> Option<&ShaderProgramScopeConfig> {
        self.scopes.iter().find(|s| s.scope_type == scope_type)
    }

    pub fn shader_ref(&self, shader_type: ShaderType) -> Option<&ShaderProgramShaderConfig> {
        self.shaders.iter().find(|s| s.shader_type == shader_type)
    }

    pub fn update_indices(&mut self) {
    }

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
    pub scope_type: ShaderScopeType,
    pub buffers: Vec<ShaderProgramBufferConfig>,
    pub samplers: Vec<ShaderProgramSamplerConfig>,
}

impl std::fmt::Display for ShaderProgramScopeConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for buffer in &self.buffers {
            writeln!(f, "\n// start: scope '{}'", &self.scope_type.struct_typedef())?;
            writeln!(f, "{}", buffer)?;
            writeln!(f, "// end: scope '{}'\n", &self.scope_type.struct_typedef())?;
        }
        Ok(())
    }
}

impl ShaderProgramScopeConfig {
    pub fn from_raw(name: &str, buffers: Vec<ShaderProgramBufferConfig>, samplers: Vec<ShaderProgramSamplerConfig>) -> HellResult<Self> {
        Ok(Self {
            scope_type: ShaderScopeType::try_from(name)?,
            buffers,
            samplers,
        })
    }

    pub fn buffer(&self, ident: &str) -> Option<&ShaderProgramBufferConfig> {
        self.buffers.iter().find(|b| b.ident == ident)
    }

    pub fn sampler(&self, ident: &str) -> Option<&ShaderProgramSamplerConfig> {
        self.samplers.iter().find(|s| s.ident == ident)
    }
}

// ----------------------------------------------------------------------------

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ShaderProgramShaderConfig {
    pub shader_type: ShaderType,
    pub uniform_usages: Vec<ShaderProgramUniformUsage>,
}

impl ShaderProgramShaderConfig {
    pub fn from_raw(shader_ident: &str, uniform_usages: Vec<ShaderProgramUniformUsage>) -> HellResult<Self> {
        Ok(Self {
            shader_type: ShaderType::try_from(shader_ident)?,
            uniform_usages,
        })
    }
}

// ----------------------------------------------------------------------------

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ShaderProgramBufferConfig {
    pub ident: String,
    pub var_ubos: Vec<ShaderProgramUboVarConfig>,
}

impl std::fmt::Display for ShaderProgramBufferConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let buffer_type_ident = format!("{}_buffer_type", self.ident);

        writeln!(f, "// START: buffer '{}'", &self.ident)?;
        writeln!(f, "layout(set = {}, binding = {}) uniform {} {{", 0, 0, buffer_type_ident)?;
        for ubo in &self.var_ubos {
            writeln!(f, "\t{}", ubo);
        }
        writeln!(f, "}} {};", &self.ident)?;
        writeln!(f, "// END: buffer '{}'", &self.ident)?;

        Ok(())
    }
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

impl std::fmt::Display for ShaderProgramUboVarConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {};", &self.type_ubo, &self.ident)
    }
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

impl std::fmt::Display for ShaderProgramSamplerConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "// START: sampler '{}'", self.ident);
        writeln!(f, "{} {};", self.type_sampler, &self.ident);
        writeln!(f, "// END: sampler '{}'", self.ident);
        Ok(())
    }
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
