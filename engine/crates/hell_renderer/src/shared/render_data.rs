use hell_error::{HellResult, ErrToHellErr};

use crate::vulkan::RenderData;

pub trait HellRenderable {
    fn get_render_data(&self) -> &RenderData;
    fn get_scene_data(&self) -> &SceneData;
}



// ----------------------------------------------------------------------------
// CameraData
// ----------------------------------------------------------------------------


#[derive(Clone)]
#[repr(C)]
pub struct CameraData {
    pub view: glam::Mat4,
    pub proj: glam::Mat4,
    pub view_proj: glam::Mat4,
}

impl CameraData {
    pub fn new(aspect_ratio: f32) -> Self {
        // let aspect_ratio = core.swapchain.aspect_ratio();

        let view = glam::Mat4::look_at_lh(glam::Vec3::new(0.0, 0.0, -2.0), glam::Vec3::new(0.0, 0.0, 0.0), glam::Vec3::new(0.0, 1.0, 0.0));
        let mut proj = glam::Mat4::perspective_lh(90.0, aspect_ratio, 0.1, 10.0);
        proj.y_axis.y *= -1.0;

        let view_proj = view * proj;

        Self {
            view, proj, view_proj
        }
    }
}

impl CameraData {
    pub fn update_view_proj(&mut self) {
        self.view_proj = self.proj * self.view;
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
