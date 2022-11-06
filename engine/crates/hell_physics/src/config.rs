pub struct PhysicsConfig {
    pub g_force: f32,
}

impl PhysicsConfig {
    pub fn new(g_force: f32) -> Self {
        Self {
            g_force,
        }
    }
}

impl Default for PhysicsConfig {
    fn default() -> Self {
        Self {
            g_force: -9.81,
        }
    }
}
