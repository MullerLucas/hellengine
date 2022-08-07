use ash::vk;
use hell_common::window::HellWindowExtent;

use super::buffer::Buffer;
use super::config;
use super::frame::FrameData;
use super::pipeline::GraphicsPipeline;
use super::render_pass::RenderPassData;
use super::vertext::Vertex;
use super::vulkan_core::Core;



static VERTICES: &[Vertex] = &[
    Vertex::from_arrays([-0.5, -0.5,  0.0, 1.0], [1.0, 0.0, 0.0, 1.0], [1.0, 0.0]),
    Vertex::from_arrays([ 0.5, -0.5,  0.0, 1.0], [0.0, 1.0, 0.0, 1.0], [0.0, 0.0]),
    Vertex::from_arrays([ 0.5,  0.5,  0.0, 1.0], [0.0, 0.0, 1.0, 1.0], [0.0, 1.0]),
    Vertex::from_arrays([-0.5,  0.5,  0.0, 1.0], [1.0, 1.0, 1.0, 1.0], [1.0, 1.0]),
];

pub static INDICES: &[u32] = &[     // u16 is also possible
    0, 1, 2,
    2, 3, 0,
];

// pub struct UniformBufferObject {
//
// }


pub struct Renderer2D  {
    pub curr_frame_idx: u32,
    pub frame_data: Vec<FrameData>,

    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,

    pipeline: GraphicsPipeline,
    render_pass_data: RenderPassData,
    core: Core,
}

impl Renderer2D {
    pub fn new(core: Core) -> Self {
        let render_pass_data = RenderPassData::new(&core);
        let pipeline = GraphicsPipeline::new(&core, &render_pass_data);
        let frame_data = FrameData::create_for_frames(&core.device.vk_device);

        let vertex_buffer = Buffer::from_vertices(&core, VERTICES);
        let index_buffer = Buffer::from_indices(&core, INDICES);


        Self {
            curr_frame_idx: 0,
            frame_data,

            vertex_buffer,
            index_buffer,

            pipeline,
            render_pass_data,
            core,
        }
    }
}

impl Drop for Renderer2D {
    fn drop(&mut self) {
        println!("> dropping Renerer2D...");

        let device = &self.core.device.vk_device;

        self.frame_data.iter().for_each(|f| {
            f.drop_manual(device);
        });

        self.vertex_buffer.drop_manual(device);
        self.index_buffer.drop_manual(device);

        self.render_pass_data.drop_manual(&self.core.device.vk_device);
        self.pipeline.drop_manual(&self.core.device.vk_device);
    }
}

impl Renderer2D {
    pub fn wait_idle(&self) {
        self.core.wait_device_idle();
    }

    pub fn on_window_changed(&mut self, window_extent: &HellWindowExtent) {
        self.core.recreate_swapchain(window_extent);
        self.render_pass_data.recreate_framebuffer(&self.core);
    }

    pub fn draw_frame(&mut self, _delta_time: f32) -> bool {
        // println!("++++++++++++++++++++++++++++++++++++++++++++++++++++++++");

        let core = &self.core;
        let device = &core.device.vk_device;
        let pipeline = &self.pipeline;
        let render_pass_data = &self.render_pass_data;

        let frame_idx = self.curr_frame_idx as usize;
        let frame_data = &self.frame_data[frame_idx];

        frame_data.wait_for_in_flight(device);

        // TODO: check
        // let swap_img_idx = match self.swapchain.aquire_next_image(frame_data.img_available_sem[0]) {
        //     Ok((idx, _)) => { idx },
        //     Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => { return true },
        //     _ => { panic!("failed to aquire next image") }
        // };
        let (swap_img_idx, _is_suboptimal) = core.swapchain.aquire_next_image(frame_data.img_available_sem[0]).unwrap();

        core.graphics_cmd_pool.reset_cmd_buffer(device, frame_idx);
        core.graphics_cmd_pool.record_cmd_buffer(core, pipeline, render_pass_data, frame_idx, swap_img_idx as usize, INDICES.len() as u32, &self.vertex_buffer, &self.index_buffer);

        // delay resetting the fence until we know for sure we will be submitting work with it (not return early)
        frame_data.reset_in_flight_fence(device);
        frame_data.submit_queue(device, core.device.queues.graphics.vk_queue, &[core.graphics_cmd_pool.get_buffer_for_frame(frame_idx)]);

        // std::thread::sleep(std::time::Duration::from_secs(1));

        let present_result = frame_data.present_queue(core.device.queues.present.vk_queue, &core.swapchain, &[swap_img_idx]);

        // TODO: check
        // do this after queue-present to ensure that the semaphores are in a consistent state - otherwise a signaled semaphore may never be properly waited upon
        let is_resized = match present_result {
            Ok(is_suboptimal) => { is_suboptimal },
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR | vk::Result::SUBOPTIMAL_KHR)  => { true },
            _ => { panic!("failed to aquire next image") }
        };

        // println!("RENDERED -FRAME: {} -- {}", self.curr_frame_idx, swap_img_idx);

        self.curr_frame_idx = (self.curr_frame_idx + 1) % config::MAX_FRAMES_IN_FLIGHT;


        is_resized
    }
}
