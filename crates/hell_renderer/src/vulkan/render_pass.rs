use ash::vk;
use hell_common::prelude::*;
use super::framebuffer::VulkanFramebuffer;
use super::image::DepthImage;
use super::vulkan_core::VulkanCore;





pub struct VulkanRenderPass {
    pub render_pass: vk::RenderPass,
}

impl VulkanRenderPass {
    pub fn new(core: &VulkanCore) -> HellResult<Self> {
        let swap_format = core.swapchain.surface_format.format;
        let msaa_samples = vk::SampleCountFlags::TYPE_1;

        // color attachments
        // -----------------
        let color_attachment = vk::AttachmentDescription::builder()
            .format(swap_format)
            .samples(msaa_samples)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .build();

        let color_attachment_refs = [
            vk::AttachmentReference::builder()
                .attachment(0) // frag-shader -> layout(location = 0
                .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .build()
        ];

        // depth attachments
        // -----------------
        let depth_attachment = vk::AttachmentDescription::builder()
            .format(core.phys_device.depth_format)
            .samples(msaa_samples)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .build();

        let depth_attachment_ref = vk::AttachmentReference::builder()
            .attachment(1)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .build();

        // subpass
        // -------
        let subpasses = [
            vk::SubpassDescription::builder()
                .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                .color_attachments(&color_attachment_refs)
                .depth_stencil_attachment(&depth_attachment_ref)
                .build()
        ];

        let subpass_dependencies = [
            vk::SubpassDependency::builder()
                .src_subpass(vk::SUBPASS_EXTERNAL)
                .dst_subpass(0)
                // operations to wait on -> wait for the swap-chain to finish reading from the img
                // depth-img is accessed first in early-fragment-test stage
                .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
                .src_access_mask(vk::AccessFlags::empty())
                // operation that has to wait: writing of the color attachment in the color attachment state
                .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS)
                // depth: we have a load-op that clears -> so we should specify the access-mask for writes
                .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE)
                .dependency_flags(vk::DependencyFlags::empty())
                .build()
        ];

        // render pass
        // -----------
        let attachments = [color_attachment, depth_attachment];

        let render_pass_info = vk::RenderPassCreateInfo::builder()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&subpass_dependencies)
            .build();

        let pass = unsafe { core.device.device.create_render_pass(&render_pass_info, None).to_render_hell_err()? };

        Ok(Self { render_pass: pass })
    }
}

impl VulkanRenderPass {
    pub fn drop_manual(&self, device: &ash::Device) {
        println!("> dropping RenderPass...");

        unsafe {
            device.destroy_render_pass(self.render_pass, None);
        }
    }
}




pub struct VulkanRenderPassData {
    pub depth_img: DepthImage,
    pub render_pass: VulkanRenderPass,
    pub framebuffer: VulkanFramebuffer,
}

impl VulkanRenderPassData {
    pub fn new(core: &VulkanCore) -> HellResult<Self> {
        let depth_img = DepthImage::new(core)?;
        let render_pass = VulkanRenderPass::new(core)?;
        let framebuffer = VulkanFramebuffer::new(&core.device.device, &core.swapchain, &render_pass, &depth_img)?;

        Ok(Self {
            depth_img,
            render_pass,
            framebuffer,
        })
    }

    pub fn recreate_framebuffer(&mut self, core: &VulkanCore) -> HellResult<()> {
        self.drop_before_recreate(&core.device.device);

        self.depth_img = DepthImage::new(core)?;
        self.framebuffer = VulkanFramebuffer::new(&core.device.device, &core.swapchain, &self.render_pass, &self.depth_img)?;

        Ok(())
    }
}

impl VulkanRenderPassData {
    pub fn drop_manual(&self, device: &ash::Device) {
        println!("> dropping RenderPassData...");

        self.depth_img.drop_manual(device);
        self.framebuffer.drop_manual(device);
        self.render_pass.drop_manual(device);
    }

    pub fn drop_before_recreate(&self, device: &ash::Device) {
        println!("> dropping RenderPassData before recreate...");

        self.depth_img.drop_manual(device);
        self.framebuffer.drop_manual(device);
    }
}
