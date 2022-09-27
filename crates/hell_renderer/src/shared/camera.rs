#[derive(Clone)]
#[repr(C)]
pub struct Camera {
    pub view: glam::Mat4,
    pub proj: glam::Mat4,
    pub view_proj: glam::Mat4,
}

impl Camera {
    pub fn new(aspect_ratio: f32) -> Self {
        // let aspect_ratio = core.swapchain.aspect_ratio();

        let view = glam::Mat4::look_at_rh(glam::Vec3::new(0.0, 0.0, 2.0), glam::Vec3::new(0.0, 0.0, 0.0), glam::Vec3::new(0.0, 1.0, 0.0));
        let proj = glam::Mat4::perspective_rh(90.0, aspect_ratio, 0.1, 10.0);
        let view_proj = view * proj;

        Self {
            view, proj, view_proj
        }
    }
}

impl Camera {
    pub fn update_view_proj(&mut self) {
        self.view_proj = self.proj * self.view;
    }
}
