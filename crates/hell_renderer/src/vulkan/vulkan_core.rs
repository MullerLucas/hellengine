use std::ffi::CString;
use std::os::raw;

use ash::prelude::VkResult;
use ash::vk;
use hell_common::window::{HellSurfaceInfo, HellWindowExtent};

use super::command_buffer::VulkanCommandPool;
use super::debugging::DebugData;
use super::frame::VulkanFrameData;
use super::logic_device::VulkanLogicDevice;
use super::phys_device::VulkanPhysDevice;
use super::pipeline::VulkanGraphicsPipeline;
use super::surface::VulkanSurface;
use super::swapchain::VulkanSwapchain;
use super::{validation_layers, platforms, debugging, config};








pub struct VulkanCore {
    pub _entry: ash::Entry,
    pub instance: ash::Instance,
    pub debug_data: DebugData,

    pub surface: VulkanSurface,

    pub phys_device: VulkanPhysDevice,
    pub device: VulkanLogicDevice,

    pub swapchain: VulkanSwapchain,

    pub graphics_cmd_pool: VulkanCommandPool,
    pub transfer_cmd_pool: VulkanCommandPool,

    // TODO: move
    pub curr_frame_idx: u32,
    pub frame_data: Vec<VulkanFrameData>,
}

impl VulkanCore {
    pub fn new(surface_info: &HellSurfaceInfo, windwow_extent: &HellWindowExtent) -> VkResult<Self> {
        let entry = unsafe { ash::Entry::load().unwrap() };
        let instance = create_instance(&entry, config::APP_NAME)?;
        let debug_data = DebugData::new(&entry, &instance);

        let surface = VulkanSurface::new(&entry, &instance, surface_info);
        let phys_device = VulkanPhysDevice::pick_phys_device(&instance, &surface);
        let device = VulkanLogicDevice::new(&instance, &phys_device);

        let graphics_cmd_pool = VulkanCommandPool::default_for_graphics(&device);
        let transfer_cmd_pool = VulkanCommandPool::default_for_transfer(&device);

        let swapchain = VulkanSwapchain::new(&instance, &phys_device, &device, &surface, windwow_extent.width, windwow_extent.height);

        // TODO: move
        let frame_data = VulkanFrameData::create_for_frames(&device.device);


        Ok(Self {
            _entry: entry,
            instance,

            surface,

            phys_device,
            device,

            graphics_cmd_pool,
            transfer_cmd_pool,

            swapchain,

            debug_data,

            curr_frame_idx: 0,
            frame_data,
        })
    }

    pub fn recreate_swapchain(&mut self, window_extent: &HellWindowExtent) {
        println!("> recreating swapchain...");

        self.swapchain.drop_manual(&self.device.device);

        let swapchain = VulkanSwapchain::new(&self.instance, &self.phys_device, &self.device, &self.surface, window_extent.width, window_extent.height);
        self.swapchain = swapchain;
    }

    pub fn wait_device_idle(&self) {
        println!("> waiting for the device to be idle...");
        self.device.wait_idle();
        println!("> done waiting for the device to be idle...");
    }
}

impl Drop for VulkanCore {
    fn drop(&mut self) {
        println!("> dropping VulkanCore");
        let device = &self.device.device;

        self.frame_data.iter().for_each(|f| {
            f.drop_manual(device);
        });

        self.graphics_cmd_pool.drop_manual(device);
        self.transfer_cmd_pool.drop_manual(device);
        self.swapchain.drop_manual(device);
        self.device.drop_manual();
        self.surface.drop_manual();
        self.debug_data.drop_manual();

        unsafe {
            self.instance.destroy_instance(None);
        }
    }
}

impl VulkanCore {
    pub fn draw_frame(&mut self, pipeline: &VulkanGraphicsPipeline, _delta_time: f32) -> bool {
        // println!("++++++++++++++++++++++++++++++++++++++++++++++++++++++++");

        let device = &self.device.device;

        let frame_idx = self.curr_frame_idx as usize;
        let frame_data = &self.frame_data[frame_idx];

        frame_data.wait_for_in_flight(&self.device.device);

        // TODO: check
        // let swap_img_idx = match self.swapchain.aquire_next_image(frame_data.img_available_sem[0]) {
        //     Ok((idx, _)) => { idx },
        //     Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => { return true },
        //     _ => { panic!("failed to aquire next image") }
        // };
        let (swap_img_idx, _is_suboptimal) = self.swapchain.aquire_next_image(frame_data.img_available_sem[0]).unwrap();

        self.graphics_cmd_pool.reset_cmd_buffer(&self.device.device, frame_idx);
        self.graphics_cmd_pool.record_cmd_buffer(self, pipeline, frame_idx, swap_img_idx as usize, config::INDICES);

        // delay resetting the fence until we know for sure we will be submitting work with it (not return early)
        frame_data.reset_in_flight_fence(device);
        frame_data.submit_queue(device, self.device.queues.graphics_queue, &[self.graphics_cmd_pool.get_buffer_for_frame(frame_idx)]);

        // std::thread::sleep(std::time::Duration::from_secs(1));

        let present_result = frame_data.present_queue(self.device.queues.present_queue, &self.swapchain, &[swap_img_idx]);

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

fn create_instance(entry: &ash::Entry, app_name: &str) -> VkResult<ash::Instance> {
    if config::ENABLE_VALIDATION_LAYERS
        && !validation_layers::check_validation_layer_support(entry, config::VALIDATION_LAYER_NAMES)
    {
        panic!("validation layers requested, but not available!");
    }


    let app_name = CString::new(app_name).unwrap();
    let engine_name = CString::new(config::ENGINE_NAME).unwrap();

    let app_info = vk::ApplicationInfo::builder()
        .application_name(&app_name)
        .engine_name(&engine_name)
        .engine_version(config::ENGINE_VERSION)
        .api_version(config::API_VERSION)
        .build();

    let extension_names = platforms::required_extension_names();

    let mut instance_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_extension_names(&extension_names)
        .build();


    // TODO: improve
    let enabled_validation_layers: Vec<_> = config::VALIDATION_LAYER_NAMES
        .iter()
        .map(|l| CString::new(*l).unwrap())
        .collect();

    let enabled_validation_layer_ref: Vec<_> = enabled_validation_layers
        .iter()
        .map(|l| l.as_ptr())
        .collect();

    let debug_utils_create_info = debugging::populate_debug_messenger_create_info();

    if config::ENABLE_VALIDATION_LAYERS {
        instance_info.enabled_layer_count = config::VALIDATION_LAYER_NAMES.len() as u32;
        instance_info.pp_enabled_layer_names = enabled_validation_layer_ref.as_ptr();
        instance_info.p_next = &debug_utils_create_info
            as *const vk::DebugUtilsMessengerCreateInfoEXT
            as *const raw::c_void;
    }

    unsafe {
        entry.create_instance(&instance_info, None)
    }
}
