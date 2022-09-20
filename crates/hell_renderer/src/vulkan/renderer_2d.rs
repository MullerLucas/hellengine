use ash::vk;
use hell_common::window::HellWindowExtent;

use super::buffer::{VulkanBuffer, VulkanUniformData};
use super::config;
use super::frame::VulkanFrameData;
use super::pipeline::VulkanGraphicsPipeline;
use super::render_pass::VulkanRenderPassData;
use super::vertext::Vertex;
use super::vulkan_core::VulkanCore;



static VERTICES: &[Vertex] = &[
    Vertex::from_arrays([-1.0, -1.0,  0.0, 1.0], [1.0, 0.0, 0.0, 1.0], [1.0, 0.0]),
    Vertex::from_arrays([ 1.0, -1.0,  0.0, 1.0], [0.0, 1.0, 0.0, 1.0], [0.0, 0.0]),
    Vertex::from_arrays([ 1.0,  1.0,  0.0, 1.0], [0.0, 0.0, 1.0, 1.0], [0.0, 1.0]),
    Vertex::from_arrays([-1.0,  1.0,  0.0, 1.0], [1.0, 1.0, 1.0, 1.0], [1.0, 1.0]),
];
// static VERTICES: &[Vertex] = &[
//     Vertex::from_arrays([-100.0, -100.0,  0.0, 1.0], [1.0, 0.0, 0.0, 1.0], [1.0, 0.0]),
//     Vertex::from_arrays([ 100.0, -100.0,  0.0, 1.0], [0.0, 1.0, 0.0, 1.0], [0.0, 0.0]),
//     Vertex::from_arrays([ 100.0,  100.0,  0.0, 1.0], [0.0, 0.0, 1.0, 1.0], [0.0, 1.0]),
//     Vertex::from_arrays([-100.0,  100.0,  0.0, 1.0], [1.0, 1.0, 1.0, 1.0], [1.0, 1.0]),
// ];

pub static INDICES: &[u32] = &[     // u16 is also possible
    0, 1, 2,
    2, 3, 0,
];

pub struct VulkanRenderer2D  {
    pub curr_frame_idx: u32,
    pub frame_data: Vec<VulkanFrameData>,

    pub vertex_buffer: VulkanBuffer,
    pub vertex_index_buffer: VulkanBuffer,

    pub uniform_data: VulkanUniformData,

    pub render_pass_data: VulkanRenderPassData,
    pub pipeline: VulkanGraphicsPipeline,
    pub core: VulkanCore,

    // pub curr_swap_idx: Option<u32>,
    // pub curr_cmd_buffer: Option<vk::CommandBuffer>,
}

impl VulkanRenderer2D {
    // TODO: error handling
    pub fn new(core: VulkanCore) -> Self {
        let frame_data = VulkanFrameData::create_for_frames(&core);

        let vertex_buffer = VulkanBuffer::from_vertices(&core, VERTICES);
        let index_buffer = VulkanBuffer::from_indices(&core, INDICES);

        // let aspect_ratio = core.swapchain.aspect_ratio();
        let uniform_data = VulkanUniformData::new(&core);

        let render_pass_data = VulkanRenderPassData::new(&core);
        let pipeline = VulkanGraphicsPipeline::new(&core, &render_pass_data, &uniform_data);



        Self {
            curr_frame_idx: 0,
            frame_data,

            vertex_buffer,
            vertex_index_buffer: index_buffer,
            uniform_data,

            pipeline,
            render_pass_data,
            core,

            // curr_swap_idx: None,
            // curr_cmd_buffer: None,
        }
    }
}

impl Drop for VulkanRenderer2D {
    fn drop(&mut self) {
        println!("> dropping Renerer2D...");

        let device = &self.core.device.device;

        self.frame_data.iter().for_each(|f| {
            f.drop_manual(device);
        });

        self.vertex_buffer.drop_manual(device);
        self.vertex_index_buffer.drop_manual(device);
        self.uniform_data.drop_manual(device);

        self.render_pass_data.drop_manual(&self.core.device.device);
        self.pipeline.drop_manual(&self.core.device.device);
    }
}

impl VulkanRenderer2D {
    pub fn wait_idle(&self) {
        self.core.wait_device_idle();
    }

    pub fn on_window_changed(&mut self, window_extent: &HellWindowExtent) {
        self.core.recreate_swapchain(window_extent);
        self.render_pass_data.recreate_framebuffer(&self.core);
    }

    pub fn draw_frame(&mut self, _delta_time: f32) -> bool {
        let core = &self.core;
        let device = &core.device.device;
        let pipeline = &self.pipeline;
        let render_pass_data = &self.render_pass_data;

        let frame_idx = self.curr_frame_idx as usize;
        let frame_data = &self.frame_data[frame_idx];
        frame_data.wait_for_in_flight(&self.core.device.device);

        // TODO: check
        // let swap_img_idx = match self.swapchain.aquire_next_image(frame_data.img_available_sem[0]) {
        //     Ok((idx, _)) => { idx },
        //     Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => { return true },
        //     _ => { panic!("failed to aquire next image") }
        // };
        let (curr_swap_idx, _is_suboptimal) = core.swapchain.aquire_next_image(frame_data.img_available_sem[0]).unwrap();

        core.graphics_cmd_pool.reset_cmd_buffer(device, frame_idx);
        core.graphics_cmd_pool.record_cmd_buffer(
            core,
            pipeline,
            render_pass_data,
            frame_idx,
            curr_swap_idx as usize,
            INDICES.len() as u32,
            &self.vertex_buffer,
            &self.vertex_index_buffer,
            &self.uniform_data
        );

        // delay resetting the fence until we know for sure we will be submitting work with it (not return early)
        frame_data.reset_in_flight_fence(device);
        frame_data.submit_queue(device, core.device.queues.graphics.queue, &[core.graphics_cmd_pool.get_buffer_for_frame(frame_idx)]);

        let present_result = frame_data.present_queue(core.device.queues.present.queue, &core.swapchain, &[curr_swap_idx]);

        // TODO: check
        // do this after queue-present to ensure that the semaphores are in a consistent state - otherwise a signaled semaphore may never be properly waited upon
        let is_resized = match present_result {
            Ok(is_suboptimal) => { is_suboptimal },
            Err(vk::Result::ERROR_OUT_OF_DATE_KHR | vk::Result::SUBOPTIMAL_KHR)  => { true },
            _ => { panic!("failed to aquire next image") }
        };

        self.curr_frame_idx = (self.curr_frame_idx + 1) % config::MAX_FRAMES_IN_FLIGHT;

        is_resized
    }
}
