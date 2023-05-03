use std::str::FromStr;

use hell_core::error::{HellError, HellErrorHelper};


// ----------------------------------------------------------------------------
// glsl-type
// ----------------------------------------------------------------------------

/// [opengl-wiki](https://www.khronos.org/opengl/wiki/Data_Type_(GLSL))
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize, PartialEq, Eq, Hash)]
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

impl TryFrom<&str> for GlslType {
    type Error = HellError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value)
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
// glsl-value
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
