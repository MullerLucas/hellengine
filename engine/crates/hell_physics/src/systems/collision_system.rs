use hell_common::transform::Transform;



pub struct CollisionSystem {
    floor_y: f32,
    ceiling_y: f32,
}

impl CollisionSystem {
    pub fn new(floor_y: f32, ceiling_y: f32) -> Self {
        Self {
            floor_y,
            ceiling_y,
        }
    }

    pub fn execute(&self, transforms: &mut [Transform]) {
        for t in transforms  {
            t.clamp_y(self.ceiling_y, self.floor_y);
        }
    }
}


impl Default for CollisionSystem {
    fn default() -> Self {
        Self::new(0.0, f32::MIN)
    }
}
