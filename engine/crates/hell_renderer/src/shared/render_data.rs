use hell_error::{HellResult, ErrToHellErr};

use crate::vulkan::RenderData;

pub trait HellRenderable {
    fn get_render_data(&self) -> &RenderData;
    fn get_scene_data(&self) -> &SceneData;
}



// ----------------------------------------------------------------------------
// NEW
// ----------------------------------------------------------------------------


#[derive(Debug, Clone)]
#[repr(C)]
pub struct ObjectUniform {
    pub model: glam::Mat4,     // 64 bytes
    pub reserve_0: glam::Mat4, // 64 bytes
    pub reserve_1: glam::Mat4, // 64 bytes
    pub reserve_2: glam::Mat4, // 64 bytes
}


// ----------------------------------------------------------------------------
// GLOBAL
// ----------------------------------------------------------------------------

//apparentyl some nvidia cards require the ubo to be 265 bytes?
#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct GlobalUniformObject {
    pub view: glam::Mat4,      // 64 bytes
    pub proj: glam::Mat4,      // 64 bytes
    pub view_proj: glam::Mat4, // 64 bytes
    pub reserve_0: glam::Mat4, // 64 bytes
}

impl GlobalUniformObject {
    pub fn new(view: glam::Mat4, proj: glam::Mat4, view_proj: glam::Mat4) -> Self {
        Self {
            view,
            proj,
            view_proj,
            reserve_0: glam::Mat4::ZERO
        }
    }
}


#[derive(Debug, Clone)]
pub struct TmpCamera {
    pub view: glam::Mat4,      // 64 bytes
    pub proj: glam::Mat4,      // 64 bytes
    pub view_proj: glam::Mat4, // 64 bytes
}

impl TmpCamera {
    // TODO:
    pub fn new(aspect_ratio: f32) -> Self {

        // let view = glam::Mat4::look_at_lh(glam::Vec3::new(0.0, 0.0, -2.0), glam::Vec3::new(0.0, 0.0, 0.0), glam::Vec3::new(0.0, 1.0, 0.0));
        let view = glam::Mat4::from_scale(glam::vec3(0.2, 0.2, 0.2));
        // let proj = glam::Mat4::IDENTITY;

        let height = 2.0;
        let width = height * aspect_ratio;
        let mut proj = glam::Mat4::orthographic_lh(
            -width / 2.0,
             width / 2.0,
            -height / 2.0,
             height / 2.0,
            0.0,
            100.0
        );
        proj.y_axis.y *= -1.0;

        let view_proj = view * proj;

        Self {
            view,
            proj,
            view_proj,
        }
    }
}




// ----------------------------------------------------------------------------
// scene data
// ----------------------------------------------------------------------------

#[repr(C)]
pub struct SceneData {
    pub tint: glam::Vec4,
    pub sun_color: glam::Vec4,
    pub sun_direction: glam::Vec4,
}


impl Default for SceneData {
    fn default() -> Self {
        Self {
            tint: glam::Vec4::new(1.0, 1.0, 1.0, 1.0),
            sun_color: glam::Vec4::new(1.0, 1.0, 1.0, 1.0),
            sun_direction: glam::Vec4::new(1.0, 1.0, 1.0, 1.0),
        }
    }
}

impl SceneData {
    pub fn update_data(&mut self) -> HellResult<()>{
        let time_raw = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH).to_generic_hell_err()?
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





// ----------------------------------------------------------------------------
// object data
// ----------------------------------------------------------------------------

#[repr(C)]
pub struct ObjectData {
    pub model: glam::Mat4,
}

impl ObjectData {
    pub const MAX_OBJ_COUNT: u64 = 10000;
}
