use hell_core::error::HellResult;

use crate::render_types::{scope::ShaderScopeType, stage::ShaderStageType, glsl::GlslType};



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

    pub fn shader_ref(&self, shader_type: ShaderStageType) -> Option<&ShaderProgramShaderConfig> {
        self.shaders.iter().find(|s| s.shader_type == shader_type)
    }

    pub fn update_sets_and_bindings(&mut self) {
        for (idx, scope) in self.scopes.iter_mut().enumerate() {
            scope.update_set_and_bindings(idx);
        }
    }
}

// ----------------------------------------------------------------------------

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ShaderProgramInfoConfig {
    pub version: String,
    pub name: String,
    pub pass: String,
}

impl ShaderProgramInfoConfig {
    pub fn from_raw(version: &str, name: &str, pass: &str) -> Self {
        Self {
            version: version.to_lowercase(),
            name: name.to_lowercase().replace("\"", ""),
            pass: pass.to_lowercase().replace("\"", ""),
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

// impl std::fmt::Display for ShaderProgramScopeConfig {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         for buffer in &self.buffers {
//             writeln!(f, "\n// start: scope '{}'", &self.scope_type.struct_typedef())?;
//             writeln!(f, "{}", buffer)?;
//             writeln!(f, "// end: scope '{}'\n", &self.scope_type.struct_typedef())?;
//         }
//         Ok(())
//     }
// }

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

    pub fn update_set_and_bindings(&mut self, set_idx: usize) {
        let mut binding = 0;

        for buffer in &mut self.buffers {
            buffer.set = set_idx;
            buffer.binding = binding;
            binding += 1;
        }

        for sampler in &mut self.samplers {
            sampler.set = set_idx;
            sampler.binding = binding;
            binding += 1;
        }
    }
}

// ----------------------------------------------------------------------------

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ShaderProgramShaderConfig {
    pub shader_type: ShaderStageType,
    pub uniform_usages: Vec<ShaderProgramUniformUsage>,
}

impl ShaderProgramShaderConfig {
    pub fn from_raw(shader_ident: &str, uniform_usages: Vec<ShaderProgramUniformUsage>) -> HellResult<Self> {
        Ok(Self {
            shader_type: ShaderStageType::try_from(shader_ident)?,
            uniform_usages,
        })
    }
}

// ----------------------------------------------------------------------------

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ShaderProgramBufferConfig {
    pub set: usize,
    pub binding: usize,
    pub ident: String,
    pub var_ubos: Vec<ShaderProgramUboVarConfig>,
}

impl ShaderProgramBufferConfig {
    pub fn from_raw(ident: &str, var_ubos: Vec<ShaderProgramUboVarConfig>) -> HellResult<Self> {
        Ok(Self {
            set: usize::MAX,
            binding: usize::MAX,
            ident: ident.to_lowercase(),
            var_ubos,
        })
    }

    pub fn format(&self, scope: ShaderScopeType, txt: &mut String) -> HellResult<()> {
        use std::fmt::Write;
        let buffer_type_ident = format!("{}_buffer_type", self.ident);
        let inner_buffer_type_ident = format!("inner_{}", buffer_type_ident);

        writeln!(txt, "// --- START: buffer '{}' ---", &self.ident)?;

        // generate start-tag
        if scope == ShaderScopeType::Local {
            writeln!(txt, "struct {} {{", inner_buffer_type_ident)?;
        } else {
            writeln!(txt, "layout(set = {}, binding = {}) uniform {} {{", self.set, self.binding, buffer_type_ident)?;
        }

        // generate members
        for ubo in &self.var_ubos {
            writeln!(txt, "\t{}", ubo)?;
        }

        // generate end-tag
        if scope == ShaderScopeType::Local {
            writeln!(txt, "\
}};
// std140 enforces cpp memory layout
layout(std140, set = {}, binding = {}) readonly buffer {} {{
    inner_{} data[];
}} {};"
            , self.set, self.binding, buffer_type_ident, inner_buffer_type_ident, self.ident)?;
        } else {
            writeln!(txt, "}} {};", &self.ident)?;
        }

        writeln!(txt, "// --- END: buffer '{}' ---", &self.ident)?;

        Ok(())
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
    pub set: usize,
    pub binding: usize,
    pub type_sampler: GlslType,
    pub ident: String,
}

impl std::fmt::Display for ShaderProgramSamplerConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "// --- START: sampler '{}' ---", self.ident)?;
        writeln!(f, "layout(set = {}, binding = {}) {} {};", self.set, self.binding, self.type_sampler, &self.ident)?;
        writeln!(f, "// --- END: sampler '{}' ---", self.ident)?;
        Ok(())
    }
}

impl ShaderProgramSamplerConfig {
    pub fn from_raw(type_sampler: &str, ident: &str) -> HellResult<Self> {
        Ok(Self {
            set: usize::MAX,
            binding: usize::MAX,
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
