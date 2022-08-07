use std::ptr;

use ash::vk;
use super::framebuffer::Framebuffer;
use super::vulkan_core::Core;





pub struct RenderPass {
    pub render_pass: vk::RenderPass,
}

impl RenderPass {
    pub fn new(core: &Core) -> Self {
        let swap_format = core.swapchain.surface_format.format;
        let msaa_samples = vk::SampleCountFlags::TYPE_1;

        // color attachments
        // -----------------
        let color_attachment = vk::AttachmentDescription {
            flags: Default::default(),
            format: swap_format,
            samples: msaa_samples,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::STORE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::PRESENT_SRC_KHR, // without multisampling
            // final_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL, // without multisampling
            // final_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL, // multisampled cannot be presented directyl -> resolve to a regular image first (does not apply to depth-buffer -> won't be presented)
        };

        let color_attachment_ref = vk::AttachmentReference {
            attachment: 0, // frag-shader -> layout(location = 0)
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };

        // let color_attachment_resolve = vk::AttachmentDescription {
        //     flags: vk::AttachmentDescriptionFlags::default(),
        //     format: swap_format,
        //     samples: vk::SampleCountFlags::TYPE_1,
        //     load_op: vk::AttachmentLoadOp::DONT_CARE,
        //     store_op: vk::AttachmentStoreOp::STORE,
        //     stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
        //     stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
        //     initial_layout: vk::ImageLayout::UNDEFINED,
        //     final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
        // };

        // instruct render-pass to resolve multisampled color image to into regular attachment
        // let color_attachment_resolve_ref = vk::AttachmentReference {
        //     attachment: 2,
        //     layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        // };




        // depth attachments
        // -----------------
        // let depth_attachment = vk::AttachmentDescription {
        //     flags: vk::AttachmentDescriptionFlags::default(),
        //     format: core.phys_device.depth_format,
        //     samples: msaa_samples,
        //     load_op: vk::AttachmentLoadOp::CLEAR,
        //     store_op: vk::AttachmentStoreOp::DONT_CARE, // we won't use the data after drawing has finished
        //     stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
        //     stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
        //     initial_layout: vk::ImageLayout::UNDEFINED, // we don't care about previous depth contents
        //     final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        // };
        //
        // let depth_attachment_ref = vk::AttachmentReference {
        //     attachment: 1,
        //     layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        // };




        // subpass
        // -------
        let subpass = vk::SubpassDescription {
            flags: Default::default(),
            pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
            input_attachment_count: 0,
            p_input_attachments: ptr::null(),
            color_attachment_count: 1,
            p_color_attachments: &color_attachment_ref,
            p_resolve_attachments: ptr::null(),//&color_attachment_resolve_ref,
            p_depth_stencil_attachment: ptr::null(), //&depth_attachment_ref,
            preserve_attachment_count: 0,
            p_preserve_attachments: ptr::null(),
        };




        // prevent transition from happening before its necessary / allowed
        // let dependencies = [vk::SubpassDependency {
        //     src_subpass: vk::SUBPASS_EXTERNAL, // refers to implicit subpass before, or after the render pass - depending on whether it is specified in src or dst
        //     dst_subpass: 0,
        //     // operations to wait on -> wait for the swap-chain to finish reading from the img
        //     // depth-img is accessed first in early-fragment-test stage
        //     src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
        //     src_access_mask: vk::AccessFlags::empty(),
        //     // operation that has to wait: writing of the color attachment in the color attachment state
        //     dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
        //         // depth: we have a load-op that clears -> so we should specify the access-mask for writes
        //     dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
        //     dependency_flags: Default::default(),
        // }];
        let subpass_dependencies = [vk::SubpassDependency {
            src_subpass: vk::SUBPASS_EXTERNAL, // refers to implicit subpass before, or after the render pass - depending on whether it is specified in src or dst
            dst_subpass: 0,
            // operations to wait on -> wait for the swap-chain to finish reading from the img
            // depth-img is accessed first in early-fragment-test stage
            src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            src_access_mask: vk::AccessFlags::empty(),
            // operation that has to wait: writing of the color attachment in the color attachment state
            dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            // depth: we have a load-op that clears -> so we should specify the access-mask for writes
            dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
            dependency_flags: vk::DependencyFlags::empty(),
        }];

        // let attachments = [color_attachment, depth_attachment, color_attachment_resolve];
        let attachments = [color_attachment];



        // render pass
        // -----------
        let render_pass_info = vk::RenderPassCreateInfo {
            s_type: vk::StructureType::RENDER_PASS_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::RenderPassCreateFlags::empty(),
            attachment_count: attachments.len() as u32,
            p_attachments: attachments.as_ptr(),
            subpass_count: 1,
            p_subpasses: &subpass,
            dependency_count: subpass_dependencies.len() as u32,
            p_dependencies: subpass_dependencies.as_ptr(),
        };

        let pass = unsafe { core.device.device.create_render_pass(&render_pass_info, None).unwrap() };


        Self { render_pass: pass }
    }
}

impl RenderPass {
    pub fn drop_manual(&self, device: &ash::Device) {
        println!("> dropping RenderPass...");

        unsafe {
            device.destroy_render_pass(self.render_pass, None);
        }
    }
}




pub struct RenderPassData {
    pub render_pass: RenderPass,
    // pub color_img: VulkanImage,
    pub framebuffer: Framebuffer,
}

impl RenderPassData {
    pub fn new(core: &Core) -> Self {
        let render_pass = RenderPass::new(core);
        // let color_img = VulkanImage::default_for_color_resource(core);
        let framebuffer = Framebuffer::new(&core.device.device, &core.swapchain, /*color_img.view,*/ &render_pass);

        Self {
            render_pass,
            // color_img,
            framebuffer,
        }
    }

    pub fn recreate_framebuffer(&mut self, core: &Core) {
        self.framebuffer.drop_manual(&core.device.device);
        let framebuffer = Framebuffer::new(&core.device.device, &core.swapchain, /*color_img.view,*/ &self.render_pass);
        self.framebuffer = framebuffer;
    }
}

impl RenderPassData {
    pub fn drop_manual(&self, device: &ash::Device) {
        println!("> dropping RenderPassData...");

        self.framebuffer.drop_manual(device);
        self.render_pass.drop_manual(device);
    }
}
