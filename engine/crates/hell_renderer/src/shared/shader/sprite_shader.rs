// ----------------------------------------------------------------------------
// uniforms
// ----------------------------------------------------------------------------

use hell_error::HellResult;

//apparentyl some nvidia cards require the ubo to be 265 bytes?
#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct SpriteShaderGlobalUniformObject {
    pub view: glam::Mat4,      // 64 bytes
    pub proj: glam::Mat4,      // 64 bytes
    pub view_proj: glam::Mat4, // 64 bytes
    pub reserve_0: glam::Mat4, // 64 bytes
}

impl SpriteShaderGlobalUniformObject {
    pub fn new(view: glam::Mat4, proj: glam::Mat4, view_proj: glam::Mat4) -> Self {
        Self {
            view,
            proj,
            view_proj,
            reserve_0: glam::Mat4::ZERO
        }
    }
}

// ----------------------------------------------

#[repr(C)]
pub struct SpriteShaderSceneData {
    pub tint: glam::Vec4,
    pub sun_color: glam::Vec4,
    pub sun_direction: glam::Vec4,
}


impl Default for SpriteShaderSceneData {
    fn default() -> Self {
        Self {
            tint: glam::Vec4::new(1.0, 1.0, 1.0, 1.0),
            sun_color: glam::Vec4::new(1.0, 1.0, 1.0, 1.0),
            sun_direction: glam::Vec4::new(1.0, 1.0, 1.0, 1.0),
        }
    }
}

impl SpriteShaderSceneData {
    pub fn update_data(&mut self) -> HellResult<()>{
        let time_raw = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)?
            .as_secs_f64();

        let time = (time_raw / 2.0 % 100_000.0) as f32;

        self.tint = glam::Vec4::new(
            time.sin(),
            time.cos(),
            time.tan(),
            1.0
        );

        Ok(())
    }
}

// ----------------------------------------------

#[repr(C)]
pub struct SpriteShaderObjectData {
    pub model: glam::Mat4,
}

impl SpriteShaderObjectData {
    pub const MAX_OBJ_COUNT: u64 = 10000;
}


// #[derive(Debug, Clone)]
// #[repr(C)]
// pub struct ObjectUniform {
//     pub model: glam::Mat4,     // 64 bytes
//     pub reserve_0: glam::Mat4, // 64 bytes
//     pub reserve_1: glam::Mat4, // 64 bytes
//     pub reserve_2: glam::Mat4, // 64 bytes
// }
