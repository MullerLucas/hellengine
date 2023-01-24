#![allow(unused)]

mod hell_pest;


use num_derive::FromPrimitive;
use num_traits::FromPrimitive;


use std::{fs, str::{Lines, FromStr}, path::Path, array};

use hell_error::{HellResult, HellError, HellErrorHelper};

/// [opengl-wiki](https://www.khronos.org/opengl/wiki/Data_Type_(GLSL))
#[derive(Debug, Clone, Copy)]
pub enum GlslType {
    // Scalars
    Bool,
    Int,
    UInt,
    Float,
    // Vectors
    BVec2,
    BVec3,
    BVec4,
    IVec2,
    IVec3,
    IVec4,
    UVec2,
    UVec3,
    UVec4,
    Vec2,
    Vec3,
    Vec4,
    DVec2,
    DVec3,
    DVec4,
    // Matrices
    Mat2,
    Mat3,
    Mat4,
    DMat2,
    DMat3,
    DMat4,
    // Opaque types
    Sampler2d,
    Sampler2dArray,
}

impl GlslType {
    pub const BOOL_PAT:  &str = "bool";
	pub const INT_PAT:   &str = "int";
	pub const UINT_PAT:  &str = "uint";
	pub const FLOAT_PAT: &str = "float";
	pub const BVEC2_PAT: &str = "bvec2";
	pub const BVEC3_PAT: &str = "bvec3";
	pub const BVEC4_PAT: &str = "bvec4";
	pub const IVEC2_PAT: &str = "ivec2";
	pub const IVEC3_PAT: &str = "ivec3";
	pub const IVEC4_PAT: &str = "ivec4";
	pub const UVEC2_PAT: &str = "uvec2";
	pub const UVEC3_PAT: &str = "uvec3";
	pub const UVEC4_PAT: &str = "uvec4";
	pub const VEC2_PAT:  &str = "vec2";
	pub const VEC3_PAT:  &str = "vec3";
	pub const VEC4_PAT:  &str = "vec4";
	pub const DVEC2_PAT: &str = "dvec2";
	pub const DVEC3_PAT: &str = "dvec3";
	pub const DVEC4_PAT: &str = "dvec4";
	pub const MAT2_PAT:  &str = "mat2";
	pub const MAT3_PAT:  &str = "mat3";
	pub const MAT4_PAT:  &str = "mat4";
	pub const DMAT2_PAT: &str = "dmat2";
	pub const DMAT3_PAT: &str = "dmat3";
	pub const DMAT4_PAT: &str = "dmat4";
	pub const SAMPLER_2D_PAT:       &str = "sampler2D";
	pub const SAMPLER_2D_ARRAY_PAT: &str = "sampler2DArray";
}

impl GlslType {
    pub fn is_sampler(&self) -> bool {
        match self {
            GlslType::Sampler2d |
            GlslType::Sampler2dArray => { true }
            _ => { false }
        }
    }
}

impl std::str::FromStr for GlslType {
    type Err = HellError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            Self::BOOL_PAT  => Ok(GlslType::Bool),
            Self::INT_PAT   => Ok(GlslType::Int),
            Self::UINT_PAT  => Ok(GlslType::UInt),
            Self::FLOAT_PAT => Ok(GlslType::Float),
            Self::BVEC2_PAT => Ok(GlslType::BVec2),
            Self::BVEC3_PAT => Ok(GlslType::BVec3),
            Self::BVEC4_PAT => Ok(GlslType::BVec4),
            Self::IVEC2_PAT => Ok(GlslType::IVec2),
            Self::IVEC3_PAT => Ok(GlslType::IVec3),
            Self::IVEC4_PAT => Ok(GlslType::IVec4),
            Self::UVEC2_PAT => Ok(GlslType::UVec2),
            Self::UVEC3_PAT => Ok(GlslType::UVec3),
            Self::UVEC4_PAT => Ok(GlslType::UVec4),
            Self::VEC2_PAT  => Ok(GlslType::Vec2),
            Self::VEC3_PAT  => Ok(GlslType::Vec3),
            Self::VEC4_PAT  => Ok(GlslType::Vec4),
            Self::DVEC2_PAT => Ok(GlslType::DVec2),
            Self::DVEC3_PAT => Ok(GlslType::DVec3),
            Self::DVEC4_PAT => Ok(GlslType::DVec4),
            Self::MAT2_PAT  => Ok(GlslType::Mat2),
            Self::MAT3_PAT  => Ok(GlslType::Mat3),
            Self::MAT4_PAT  => Ok(GlslType::Mat4),
            Self::DMAT2_PAT => Ok(GlslType::DMat2),
            Self::DMAT3_PAT => Ok(GlslType::DMat3),
            Self::DMAT4_PAT => Ok(GlslType::DMat4),
            Self::SAMPLER_2D_PAT       => Ok(GlslType::Sampler2d),
            Self::SAMPLER_2D_ARRAY_PAT => Ok(GlslType::Sampler2dArray),
            _ => Err(HellErrorHelper::render_msg_err("failed to parse glsl-type"))
        }
    }
}

impl GlslType {
    pub fn to_str(&self) -> &str {
        match self {
            GlslType::Bool  => Self::BOOL_PAT,
            GlslType::Int   => Self::INT_PAT,
            GlslType::UInt  => Self::UINT_PAT,
            GlslType::Float => Self::FLOAT_PAT,
            GlslType::BVec2 => Self::BVEC2_PAT,
            GlslType::BVec3 => Self::BVEC3_PAT,
            GlslType::BVec4 => Self::BVEC4_PAT,
            GlslType::IVec2 => Self::IVEC2_PAT,
            GlslType::IVec3 => Self::IVEC3_PAT,
            GlslType::IVec4 => Self::IVEC4_PAT,
            GlslType::UVec2 => Self::UVEC2_PAT,
            GlslType::UVec3 => Self::UVEC3_PAT,
            GlslType::UVec4 => Self::UVEC4_PAT,
            GlslType::Vec2  => Self::VEC2_PAT,
            GlslType::Vec3  => Self::VEC3_PAT,
            GlslType::Vec4  => Self::VEC4_PAT,
            GlslType::DVec2 => Self::DVEC2_PAT,
            GlslType::DVec3 => Self::DVEC3_PAT,
            GlslType::DVec4 => Self::DVEC4_PAT,
            GlslType::Mat2  => Self::MAT2_PAT,
            GlslType::Mat3  => Self::MAT3_PAT,
            GlslType::Mat4  => Self::MAT4_PAT,
            GlslType::DMat2 => Self::DMAT2_PAT,
            GlslType::DMat3 => Self::DMAT3_PAT,
            GlslType::DMat4 => Self::DMAT4_PAT,
            GlslType::Sampler2d      => Self::SAMPLER_2D_PAT,
            GlslType::Sampler2dArray => Self::SAMPLER_2D_ARRAY_PAT,
        }
    }
}

impl std::fmt::Display for GlslType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

// ----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum GlslValue {
    // Scalars
    Bool(bool),
    Int(i32),
    UInt(u32),
    Float(f32),
    // Vectors
    BVec2(glam::BVec2),
    BVec3(glam::BVec3),
    BVec4(glam::BVec4),
    IVec2(glam::IVec2),
    IVec3(glam::IVec3),
    IVec4(glam::IVec4),
    UVec2(glam::UVec2),
    UVec3(glam::UVec3),
    UVec4(glam::UVec4),
    Vec2(glam::Vec2),
    Vec3(glam::Vec3),
    Vec4(glam::Vec4),
    DVec2(glam::DVec2),
    DVec3(glam::DVec3),
    DVec4(glam::DVec4),
    // Matrices
    Mat2(glam::Mat2),
    Mat3(glam::Mat3),
    Mat4(glam::Mat4),
    DMat2(glam::DMat2),
    DMat3(glam::DMat3),
    DMat4(glam::DMat4),
    // Opaque types
    Sampler2d(u32),
    Sampler2dArray(u32),
}

// ----------------------------------------------------------------------------

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct GlslVarDeclaration<'a> {
    pub var_type: GlslType,
    pub var_name: &'a str,
    pub var_val: Option<GlslValue>
}

impl<'a> GlslVarDeclaration<'a> {
    pub fn parse_txt(txt: &'a str) -> Option<Self> {
        if let Some((var_type_raw, var_name)) = txt.trim_start().split_once(" ") {
            let var_type = GlslType::from_str(var_type_raw).ok()?;
            let var_name = var_name.trim_end_matches(';');

            return Some(Self {
                var_type,
                var_name,
                var_val: None,
            });
        }

        None
    }
}

impl<'a> std::fmt::Display for GlslVarDeclaration<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {};", self.var_type, self.var_name)
    }
}

// ----------------------------------------------------------------------------

#[derive(Default, Debug, Clone, Copy, FromPrimitive)]
enum ShaderScopeType {
    #[default]
    Global = 0,
    Shared,
    Instance,
    Local
}

impl ShaderScopeType {
    pub const SCOPE_COUNT: usize = 4;

    pub fn struct_name(&self) -> &str {
        match self {
            ShaderScopeType::Global   => "GlobalUbo",
            ShaderScopeType::Shared   => "SharedUbo",
            ShaderScopeType::Instance => "InstanceUbo",
            ShaderScopeType::Local    => "LocalUbo"
        }
    }

    pub fn struct_typedef(&self) -> &str {
        match self {
            ShaderScopeType::Global   => "global",
            ShaderScopeType::Shared   => "shared",
            ShaderScopeType::Instance => "instance",
            ShaderScopeType::Local    => "local"
        }
    }
}

impl ShaderScopeType {
    fn parse_txt(txt: &str) -> Option<Self> {
        match txt.trim() {
            "global"   => Some(Self::Global),
            "shared"   => Some(Self::Shared),
            "instance" => Some(Self::Instance),
            "local"    => Some(Self::Local),
            _ => None,
        }
    }
}

// ----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
#[repr(usize)]
enum ShaderType {
    Vertex = 0,
    Fragment
}

impl ShaderType {
    pub const SHADER_TYPE_COUNT: usize = 2;

    fn parse_txt(txt: &str) -> Option<Self> {
        println!("SHADERTYPE: '{}'", txt);
        match txt.trim_start() {
            "vert"|"vertex"   => Some(Self::Vertex),
            "frag"|"fragment" => Some(Self::Fragment),
            _ => None,
        }
    }
}

// ----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
enum HellInstruction {
    DefineScope(ShaderScopeType),
    UseScope(ShaderScopeType),
    DefineShader(ShaderType),
    // DefineUniform(&'str),
}

impl HellInstruction {
    pub fn define_scope(after: &str) -> Option<Self> {
        ShaderScopeType::parse_txt(after).and_then(|s| Some(Self::DefineScope(s)))
    }

    pub fn use_scope(after: &str) -> Option<Self> {
        ShaderScopeType::parse_txt(after).and_then(|s| Some(Self::UseScope(s)))
    }

    pub fn define_shader(after: &str) -> Option<Self> {
        ShaderType::parse_txt(after).and_then(|s| Some(Self::DefineShader(s)))
    }
}

// ----------------------------------------------------------------------------

impl HellInstruction {
    pub fn parse_txt(txt: &str) -> Option<HellInstruction> {
        const PATTERNS: &[&str] = &[
            "scope",
            "use-scope",
            "shader",
        ];

        const GENERATORS: &[fn(&str) -> Option<HellInstruction>] = &[
            HellInstruction::define_scope,
            HellInstruction::use_scope,
            HellInstruction::define_shader,
        ];

        let txt = txt.trim_start();
        println!("parse-instr: '{}'", txt);

        for (pat, gen) in PATTERNS.iter().zip(GENERATORS) {
            let res = Self::check_txt(*pat, txt, *gen);
            if res.is_some() { return res; }
        }

        None
    }

    fn check_txt(pat: &str, txt: &str, gen: fn(&str) -> Option<Self>) -> Option<Self> {
        if txt.starts_with(pat) {
            gen(&txt[pat.len()..])
        } else {
            None
        }
    }
}

// ----------------------------------------------------------------------------

#[derive(Debug)]
enum HellLine<'a> {
    Invalid(&'a str),
    Instruction(HellInstruction),
    UniformDeclaration(GlslVarDeclaration<'a>),
    PlainText(&'a str),
    Comment(&'a str),
}

impl<'a> HellLine<'a> {
    pub fn parce_line(line: &'a str) -> Self {
        const INSTR_PAT: &str = "#hell";
        const COMMENT_PAT: &str = "//";

        let line = line.trim();

        if line.starts_with(INSTR_PAT) {
            if let Some(instr) = HellInstruction::parse_txt(&line[INSTR_PAT.len()..]) {
                return HellLine::Instruction(instr);
            }
            return HellLine::Invalid(line);
        }

        if let Some(decl) = GlslVarDeclaration::parse_txt(line) {
            return HellLine::UniformDeclaration(decl);
        }

        if line.starts_with(COMMENT_PAT) {
        return HellLine::Comment(&line[COMMENT_PAT.len()..]);
    }

        HellLine::PlainText(line)
    }
}

struct HellLineIter<'a> {
    lines: Lines<'a>,
}

impl<'a> From<&'a str> for HellLineIter<'a> {
    fn from(val: &'a str) -> Self {
        Self { lines: val.lines() }
    }
}

impl<'a> Iterator for HellLineIter<'a> {
    type Item = HellLine<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.lines.next().and_then(|l| {
            Some(HellLine::parce_line(l))
        })
    }
}


// ----------------------------------------------------------------------------


#[derive(Debug)]
struct ScopeInfo<'a> {
    pub is_used: bool,
    pub scope_type: ShaderScopeType,
    pub set: usize,
    pub binding: usize,
    pub uniforms: Vec<GlslVarDeclaration<'a>>,
}

impl<'a> ScopeInfo<'a> {
    pub fn new(scope_type: ShaderScopeType) -> Self {
        Self {
            is_used: false,
            scope_type,
            set: 0,
            binding: 0,
            uniforms: Vec::new(),
        }
    }

    pub fn finalize(&mut self, set_idx: usize) {
        self.set = set_idx;
        // self.uniforms.iter_mut()
        //     .filter(|u| u.var_type.is_sampler())
        //     .enumerate()
        //     .for_each(|(idx, u)| {
        //         u.bin
        //     })
    }
}

impl<'a> std::fmt::Display for ScopeInfo<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_used {
            writeln!(f, "layout(set = {}, binding = {}) {} {{", self.set, self.binding, self.scope_type.struct_name())?;
            for uniform in &self.uniforms {
                writeln!(f, "\t{}", uniform)?;
            }
            writeln!(f, "}} {};", self.scope_type.struct_typedef())?;
        }

        Ok(())
    }
}

// ----------------------------------------------------------------------------

#[allow(dead_code)]
#[derive(Debug, Default)]
struct ShaderInfo {
    pub is_used: bool,
    pub body: String,
}

impl std::fmt::Display for ShaderInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.body)
    }
}


// ----------------------------------------------------------------------------

struct ShaderParser<'a> {
    lines: HellLineIter<'a>,
    scopes: [ScopeInfo<'a>; ShaderScopeType::SCOPE_COUNT],
    shaders: [ShaderInfo; ShaderType::SHADER_TYPE_COUNT],
}

impl<'a> ShaderParser<'a> {
    pub fn new(raw: &'a str) -> Self {
        let lines = HellLineIter::from(raw);
        let scopes = array::from_fn(|idx| {
            ScopeInfo::new(ShaderScopeType::from_usize(idx).unwrap())
        });
        let shaders = Default::default();

        Self {
            lines,
            scopes,
            shaders,
        }
    }

    pub fn process(&mut self) -> HellResult<()> {
        while let Some(line) = self.lines.next() {
            println!("{:?}", line);
            match line {
                HellLine::Instruction(instr) => {
                    match instr {
                        HellInstruction::DefineScope(scope_type)   => { self.process_define_scope(scope_type); }
                        HellInstruction::DefineShader(shader_type) => { self.process_define_shader(shader_type); }
                        _ => { },
                    }
                }
                _ => { }
            }
        }

        Ok(())
    }

    pub fn finalize(&mut self) {
        let mut scope_idx = 0;

        for scope in &mut self.scopes {
            if !scope.is_used { continue; }
            scope.finalize(scope_idx);
            scope_idx += 1;
        }
    }

    pub fn write_to_file(&self) -> HellResult<()> {
        self.write_shader(Path::new("shaders/out/parse.vert"), ShaderType::Vertex)?;
        self.write_shader(Path::new("shaders/out/parse.frag"), ShaderType::Fragment)?;

        Ok(())
    }
}

impl<'a> ShaderParser<'a> {
    pub fn write_shader(&self, path: &Path, scope_type: ShaderType) -> HellResult<()> {
        use std::fmt::Write;
        let mut buff = String::new();

        for scope in &self.scopes {
            if !scope.is_used { continue; }

            writeln!(buff, "{}", scope)?;
        }

        let shader = &self.scopes[scope_type as usize];
        writeln!(buff, "{}", shader)?;

        fs::write(path, buff)?;

        Ok(())
    }

    fn process_define_scope(&mut self, scope: ShaderScopeType) {
        let scope = &mut self.scopes[scope as usize];
        scope.is_used = true;

        while let Some(line) = &self.lines.next() {
            match line {
                HellLine::Instruction(HellInstruction::DefineScope(_)) => { return; },
                HellLine::UniformDeclaration(decl) => {
                    scope.uniforms.push(decl.clone())
                }
                _ => { }
            }
        }
    }

    fn process_define_shader(&mut self, shader_type: ShaderType) {
        let shader = &mut self.shaders[shader_type as usize];

        while let Some(line) = &self.lines.next() {
            println!("{:?}", line);
            match line {
                HellLine::Instruction(HellInstruction::DefineShader(_)) => { return; },
                HellLine::PlainText(txt) => {
                    shader.body.push_str(&format!("{}\n", txt));
                }
                _ => { }
            }
        }
    }
}

// ------------------------

#[derive(Debug)]
pub enum ShaderKeyword {
    GlslLayout,
}



// ----------------------------------------------------------------------------

fn main() {
    // let raw = fs::read_to_string("shaders/parse.glsl").unwrap();
    // let mut parser = ShaderParser::new(&raw);
    // parser.process().unwrap();
    // parser.finalize();
    // parser.write_to_file().unwrap();

    // let in_buff = fs::read_to_string(Path::new("shaders/parse.glsl")).unwrap();
    // let _abt: AbstractSyntaxTree<ShaderKeyword, GlslType> = AbstractSyntaxTree::parse_buff(&in_buff).unwrap();

    hell_pest::run().unwrap();
    print!("done");
}
