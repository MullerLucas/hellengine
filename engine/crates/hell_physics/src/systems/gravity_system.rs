use hell_common::transform::Transform;

use crate::PhysicsConfig;

#[derive(Default)]
pub struct GravitySystem {
    config: PhysicsConfig
}

impl GravitySystem {
    pub fn new(config: PhysicsConfig) -> Self {
        Self {
            config
        }
    }

    pub fn execute(&self, transforms: &mut [Transform], delta_time: f32) {
        let offset = self.config.g_force * delta_time;

        for t in transforms {
            t.translate_y(offset);
        }
    }
}
