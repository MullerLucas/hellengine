use glam::Mat4;

//apparentyl some nvidia cards require the ubo to be 265 bytes?
#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct CameraUniform {
    pub view: Mat4,      // 64 bytes
    pub proj: Mat4,      // 64 bytes
    pub view_proj: Mat4, // 64 bytes
    pub reserve_0: Mat4, // 64 bytes
}

impl CameraUniform {
    pub fn new(view: Mat4, proj: Mat4, view_proj: Mat4) -> Self {
        Self {
            view,
            proj,
            view_proj,
            reserve_0: Mat4::ZERO
        }
    }
}

// ----------------------------------------------

