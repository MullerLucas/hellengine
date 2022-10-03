use hell_renderer::render_data::SceneData;
use hell_renderer::vulkan::RenderData;





#[derive(Default)]
pub struct Scene {
    scene_data: SceneData,
    render_data: RenderData,
}


impl Scene {
    pub fn new() -> Scene {
        let scene_data = SceneData::default();
        let render_data = RenderData::default();

        Scene {
            scene_data,
            render_data,
        }
    }

}

impl Scene {
    pub fn get_scene_data_mut(&mut self) -> &mut SceneData {
        &mut self.scene_data
    }

    pub fn get_render_data(&self) -> &RenderData {
        &self.render_data
    }

    pub fn get_render_data_mut(&mut self) -> &mut RenderData {
        &mut self.render_data
    }
}
