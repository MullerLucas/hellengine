// ----------------------------------------------------------------------------
// uniforms
// ----------------------------------------------------------------------------

#[repr(C)]
pub struct BmFontShaderUniform {
    pub model: glam::Mat4,
    pub reserved_0: glam::Mat4, // 64 bytes
    pub reserved_1: glam::Mat4, // 64 bytes
    pub reserved_2: glam::Mat4, // 64 bytes
}

impl BmFontShaderUniform {
    pub const MAX_OBJ_COUNT: u64 = 10000;

    pub fn new(model: glam::Mat4) -> Self {
        Self {
            model,
            reserved_0: glam::Mat4::ZERO,
            reserved_1: glam::Mat4::ZERO,
            reserved_2: glam::Mat4::ZERO,
        }
    }
}
