use glam::{Vec2, Vec4};
use hell_common::transform::Transform;

#[derive(Debug, Clone)]
pub struct AABB2D {
    pub min: Vec2,
    pub max: Vec2,
}

impl Default for AABB2D {
    fn default() -> Self {
        Self {
            min: glam::vec2(-1.0, -1.0),
            max: glam::vec2(1.0, 1.0),
        }
    }
}

impl AABB2D {
    pub fn transform(&self, t: &Transform) -> Self {
        let model = t.create_model_mat();
        let min = model * Vec4::from((self.min, 0.0, 1.0));
        let max = model * Vec4::from((self.max, 0.0, 1.0));

        Self {
            min: glam::vec2(min.x, min.y),
            max: glam::vec2(max.x, max.y),
        }
    }

    pub fn does_overlap(&self, other: &AABB2D) -> bool {
        for i in 0..2 {
            if (self.max[i] < other.min[i]) ||
               (self.min[i] > other.max[i])
               { return false; }
        }

        true
    }
}
