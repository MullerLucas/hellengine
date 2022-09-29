use hell_common::transform::Transform;
use hell_renderer::render_data::SceneData;
use hell_renderer::vulkan::RenderData;

pub struct Scene {
    scene_data: SceneData,
    render_data: RenderData,
}

impl Scene {
    pub fn new() -> Self {
        let mut render_data = RenderData::default();
        render_data.add_data(0, 0, Transform::default());
        render_data.add_data(0, 1, Transform::default());
        render_data.add_data(0, 2, Transform::default());

        let scene_data = SceneData::default();

        Self {
            scene_data,
            render_data
        }
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene {
    pub fn get_scene_data_mut(&mut self) -> &mut SceneData {
        &mut self.scene_data
    }

    pub fn get_render_data(&self) -> &RenderData {
        &self.render_data
    }

    pub fn update(&mut self, delta_time: f32) {
        let trans_1 = &mut self.render_data.transforms[0];
        trans_1.scale_uniform(1f32 + delta_time / 5f32);
        trans_1.rotate_around_z((delta_time * 30f32).to_radians());
        trans_1.translate_x(delta_time);

        let trans_2 = &mut self.render_data.transforms[1];
        trans_2.scale_uniform(1f32 - delta_time / 5f32);
        trans_2.rotate_around_z((delta_time * -30f32).to_radians());
        trans_2.translate_x(-delta_time);
    }
}
