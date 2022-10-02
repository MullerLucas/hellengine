use crate::vulkan::RenderData;

pub trait Renderable {
    fn get_render_data(&self) -> RenderData;
}
