use glam::{Mat4, Vec3, Quat};

#[derive(Debug, Clone)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale:  Vec3,
}

impl Transform {
    pub const IDENTITY: Transform = Transform::identity();
}

impl Transform {
    pub const fn new(translation: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self { translation, rotation, scale }
    }

    pub const fn identity() -> Self {
        Self::new(
            Vec3::ZERO,
            Quat::IDENTITY,
            Vec3::ONE
        )
    }
}

impl Default for Transform {
    fn default() -> Self {
        Self::identity()
    }
}

impl Transform {
    pub fn create_model_mat(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            self.scale,
            self.rotation,
            self.translation
        )
    }
}



// translations
// ------------
impl Transform {
    pub fn translate(&mut self, offset: Vec3) {
        self.translation += offset;
    }

    pub fn translate_x(&mut self, offset: f32) {
        self.translation.x += offset;
    }

    pub fn translate_y(&mut self, offset: f32) {
        self.translation.y += offset;
    }

    pub fn translate_z(&mut self, offset: f32) {
        self.translation.z += offset;
    }

    pub fn translate_xy(&mut self, offset: glam::Vec2) {
        self.translation += glam::Vec3::new(offset.x, offset.y, 0.0);
    }

    pub fn clamp_x(&mut self, min: f32, max: f32) {
        self.translation.x = self.translation.x.clamp(min, max);
    }

    pub fn clamp_y(&mut self, min: f32, max: f32) {
        self.translation.y = self.translation.y.clamp(min, max);
    }

    pub fn clamp_z(&mut self, min: f32, max: f32) {
        self.translation.z = self.translation.z.clamp(min, max);
    }
}

// rotations
// ---------
impl Transform {
    pub fn rotate(&mut self, offset: Quat) {
        self.rotation = offset * self.rotation;
    }

    pub fn rotate_around(&mut self, axis: Vec3, angle: f32) {
        self.rotate(Quat::from_axis_angle(axis, angle));
    }

    pub fn rotate_around_x(&mut self, angle: f32) {
        self.rotate(Quat::from_rotation_x(angle));
    }

    pub fn rotate_around_y(&mut self, angle: f32) {
        self.rotate(Quat::from_rotation_y(angle));
    }

    pub fn rotate_around_z(&mut self, angle: f32) {
        self.rotate(Quat::from_rotation_z(angle));
    }
}


// scale
// -----

impl Transform {
    pub fn scale_uniform(&mut self, factor: f32) {
        self.scale *= factor;
    }
}
