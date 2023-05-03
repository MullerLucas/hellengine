use ash::vk;
use hell_core::collections::dyn_array::DynArray;
use hell_core::error::HellResult;

use crate::vulkan::VulkanContextRef;

use super::{VulkanSwapchain, VulkanImage, VulkanFramebuffer, VulkanCommands};



bitflags::bitflags! {
    pub struct RenderPassClearFlags: u32 {
        const NONE          = 0x00;
        const COLORBUFFER   = 0x01;
        const DEPTHBUFFER   = 0x02;
        const STENCILBUFFER = 0x04;
    }
}


pub enum BultinRenderPassType {
    World,
    Ui,
}

pub struct VulkanRenderPass {
    ctx: VulkanContextRef,
    pub handle: vk::RenderPass,

    pub depth: f32,
    pub stencil: u32,

    pub clear_flags: RenderPassClearFlags,
    pub has_next: bool,
    pub has_prev: bool,
}

impl Drop for VulkanRenderPass {
    fn drop(&mut self) {
        println!("> dropping RenderPass...");

        unsafe {
            let device = &self.ctx.device.handle;
            device.destroy_render_pass(self.handle, None);
        }
    }
}

impl VulkanRenderPass {
    pub fn new(ctx: &VulkanContextRef, swapchain: &VulkanSwapchain, clear_flags: RenderPassClearFlags, has_prev: bool, has_next: bool) -> HellResult<Self> {
        let swap_format = swapchain.surface_format.format;
        let msaa_samples = vk::SampleCountFlags::TYPE_1;

        // subpass start
        // -------------
        let mut subpass = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS);

        // attachments
        // -----------
        const MAX_ATTACHMENT_COUNT: usize = 2;
        let mut attachments = DynArray::<vk::AttachmentDescription, MAX_ATTACHMENT_COUNT>::default();

        // color attachments
        // -----------------
        let color_attachment_load_op        = if clear_flags.contains(RenderPassClearFlags::COLORBUFFER) { vk::AttachmentLoadOp::CLEAR } else { vk::AttachmentLoadOp::LOAD };
        let color_attachment_initial_layout = if has_prev { vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL } else { vk::ImageLayout::UNDEFINED };
        let color_attachment_final_layout   = if has_next { vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL } else { vk::ImageLayout::PRESENT_SRC_KHR };

        let color_attachment = vk::AttachmentDescription::builder()
            .format(swap_format)
            .samples(msaa_samples)
            .load_op(color_attachment_load_op)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(color_attachment_initial_layout)
            .final_layout(color_attachment_final_layout)
            .build();

        let color_attachment_refs = [
            vk::AttachmentReference::builder()
                .attachment(0) // frag-shader -> layout(location = 0
                .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                .build()
        ];

        attachments.push(color_attachment);
        subpass = subpass.color_attachments(&color_attachment_refs);

        // depth attachments
        // -----------------
        let should_clear_depth = clear_flags.contains(RenderPassClearFlags::DEPTHBUFFER);
        let use_depth = should_clear_depth; // TODO:

        #[allow(unused)] let mut depth_attachment = Default::default();
        #[allow(unused)] let mut depth_attachment_ref = Default::default();

        if use_depth {

            let depth_attachment_load_op = if should_clear_depth { vk::AttachmentLoadOp::CLEAR } else { vk::AttachmentLoadOp::LOAD };

            depth_attachment = vk::AttachmentDescription::builder()
                .format(ctx.phys_device.depth_format)
                .samples(msaa_samples)
                .load_op(depth_attachment_load_op)
                .store_op(vk::AttachmentStoreOp::DONT_CARE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .build();

            depth_attachment_ref = vk::AttachmentReference::builder()
                .attachment(1)
                .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .build();

            attachments.push(depth_attachment);
            subpass = subpass.depth_stencil_attachment(&depth_attachment_ref);
        }


        // subpass start
        // -------------
        let subpasses = [subpass.build()];

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

        let render_pass_info = vk::RenderPassCreateInfo::builder()
            .attachments(attachments.as_slice())
            .subpasses(&subpasses)
            .dependencies(&subpass_dependencies)
            .build();

        let pass = unsafe { ctx.device.handle.create_render_pass(&render_pass_info, None)? };

        Ok(Self {
            ctx: ctx.clone(),
            handle: pass,

            depth: 1.0,
            stencil: 0,

            clear_flags,
            has_next,
            has_prev,
        })
    }
}



// ----------------------------------------------------------------------------
// Render-Pass-Data
// ----------------------------------------------------------------------------

pub struct VulkanRenderPassData {
    pub world_depth_img: VulkanImage,
    pub world_framebuffer: VulkanFramebuffer,
    pub world_render_pass: VulkanRenderPass,

    pub ui_framebuffer: VulkanFramebuffer,
    pub ui_render_pass: VulkanRenderPass,
}

impl VulkanRenderPassData {
    pub fn new(ctx: &VulkanContextRef, swapchain: &VulkanSwapchain, cmds: &VulkanCommands) -> HellResult<Self> {
        let world_clear_flags = RenderPassClearFlags::COLORBUFFER | RenderPassClearFlags::DEPTHBUFFER | RenderPassClearFlags::STENCILBUFFER;
        let world_depth_img = VulkanImage::new_depth_img(ctx, swapchain, cmds)?;
        let world_render_pass = VulkanRenderPass::new(ctx, swapchain, world_clear_flags, false, true)?;
        let world_framebuffer = VulkanFramebuffer::new_world_buffer(ctx, swapchain, &world_render_pass, &world_depth_img)?;

        let ui_clear_flags = RenderPassClearFlags::NONE;
        let ui_render_pass = VulkanRenderPass::new(ctx, swapchain, ui_clear_flags, true, false)?;
        let ui_framebuffer = VulkanFramebuffer::new_ui_buffer(ctx, swapchain, &ui_render_pass)?;

        Ok(Self {
            world_render_pass,
            world_depth_img,
            world_framebuffer,

            ui_framebuffer,
            ui_render_pass,
        })
    }

    pub fn recreate_framebuffer(&mut self, ctx: &VulkanContextRef, swapchain: &VulkanSwapchain, cmds: &VulkanCommands) -> HellResult<()> {
        self.world_depth_img = VulkanImage::new_depth_img(ctx, swapchain, cmds)?;
        self.world_framebuffer = VulkanFramebuffer::new_world_buffer(ctx, swapchain, &self.world_render_pass, &self.world_depth_img)?;

        self.ui_framebuffer = VulkanFramebuffer::new_ui_buffer(ctx, swapchain, &self.ui_render_pass)?;

        Ok(())
    }
}
