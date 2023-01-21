// ----------------------------------------------------------------------------
// Camera
// ----------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct HellCamera {
    pub view: glam::Mat4,      // 64 bytes
    pub proj: glam::Mat4,      // 64 bytes
    pub view_proj: glam::Mat4, // 64 bytes
}

impl HellCamera {
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
