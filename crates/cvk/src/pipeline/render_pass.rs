use ash::vk;
use utils::{Build, Buildable};

use crate::{Context, Format, ImageAspect, ImageLayout, infer_aspect_from_format};

pub type AttachmentLoadOp = vk::AttachmentLoadOp;
pub type AttachmentStoreOp = vk::AttachmentStoreOp;

#[derive(utils::Paramters, Copy, Clone, Debug)]
pub struct Attachment {
    pub format: Format,
    pub load_op: AttachmentLoadOp,
    pub store_op: AttachmentStoreOp,
    pub initial_layout: ImageLayout,
    pub final_layout: ImageLayout,
}

impl Attachment {
    pub fn color(format: Format) -> Self {
        assert!(infer_aspect_from_format(format).contains(ImageAspect::COLOR), "The given format cannot be used as in a color attachment");
        Self {
            format,
            load_op: AttachmentLoadOp::CLEAR,
            store_op: AttachmentStoreOp::STORE,
            initial_layout: ImageLayout::UNDEFINED,
            final_layout: ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        }
    }

    pub fn depth(format: Format) -> Self {
        assert!(infer_aspect_from_format(format).contains(ImageAspect::DEPTH), "The given format cannot be used as in a depth attachment");
        Self {
            format,
            load_op: AttachmentLoadOp::DONT_CARE,
            store_op: AttachmentStoreOp::DONT_CARE,
            initial_layout: ImageLayout::UNDEFINED,
            final_layout: ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        }
    }

    pub fn swapchain() -> Self {
        let format = Context::get().swapchain().expect("Failed to create swapchain attachment, because no swapchain is present").format().format;

        assert!(infer_aspect_from_format(format).contains(ImageAspect::COLOR), "The given format cannot be used as in a color attachment");

        Self {
            format,
            load_op: AttachmentLoadOp::DONT_CARE,
            store_op: AttachmentStoreOp::STORE,
            initial_layout: ImageLayout::UNDEFINED,
            final_layout: ImageLayout::PRESENT_SRC_KHR,
        }
    }
}

impl Default for Attachment {
    fn default() -> Self {
        Self {
            format: Format::UNDEFINED,
            load_op: AttachmentLoadOp::DONT_CARE,
            store_op: AttachmentStoreOp::DONT_CARE,
            initial_layout: ImageLayout::UNDEFINED,
            final_layout: ImageLayout::UNDEFINED,
        }
    }
}

#[derive(cvk_macros::VkHandle, utils::Share, Debug)]
pub struct RenderPass {
    handle: vk::RenderPass,

    attachments: Vec<Attachment>,
}

impl RenderPass {
    #[inline]
    pub const fn attachments(&self) -> &Vec<Attachment> {
        &self.attachments
    }
}

impl Drop for RenderPass {
    fn drop(&mut self) {
        unsafe {
            Context::get_device().destroy_render_pass(self.handle, None);
        }
    }
}

impl Buildable for RenderPass {
    type Builder<'a> = RenderPassBuilder;
}

#[derive(utils::Paramters, Debug, Clone)]
pub struct RenderPassBuilder {
    #[vec(push_attachment)]
    attachments: Vec<Attachment>,
}

impl Default for RenderPassBuilder {
    fn default() -> Self {
        Self {
            attachments: vec![],
        }
    }
}

impl Build for RenderPassBuilder {
    type Target = RenderPass;

    fn build(&self) -> Self::Target {
        let mut attachment_descriptions = vec![];
        let mut color_refs = vec![];
        let mut depth_ref = None;

        for attachment in self.attachments.iter() {
            attachment_descriptions.push(
                vk::AttachmentDescription::default()
                    .format(attachment.format)
                    .load_op(attachment.load_op)
                    .store_op(attachment.store_op)
                    .initial_layout(attachment.initial_layout)
                    .final_layout(attachment.final_layout)
                    .stencil_load_op(AttachmentLoadOp::DONT_CARE)
                    .stencil_store_op(AttachmentStoreOp::DONT_CARE)
                    .samples(vk::SampleCountFlags::TYPE_1)
            );

            let index = (attachment_descriptions.len() - 1) as u32;
            let aspect = infer_aspect_from_format(attachment.format);

            let layout = if aspect.contains(ImageAspect::COLOR) {
                ImageLayout::COLOR_ATTACHMENT_OPTIMAL
            } else {
                ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
            };

            let attachment_ref = vk::AttachmentReference::default()
                .attachment(index)
                .layout(layout);

            if aspect.contains(ImageAspect::COLOR) {
                color_refs.push(attachment_ref);
            } else if depth_ref.is_none() {
                depth_ref = Some(attachment_ref);
            } else {
                panic!("Only one depth attachment is allowed");
            }
        }

        let mut subpass_description = vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color_refs);

        if let Some(depth_ref) = depth_ref.as_ref() {
            subpass_description = subpass_description.depth_stencil_attachment(depth_ref);
        }

        let stage_mask = vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS;
        let access_mask = vk::AccessFlags::COLOR_ATTACHMENT_WRITE | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE;

        let subpass_dependency = vk::SubpassDependency::default()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .src_stage_mask(stage_mask)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_subpass(0)
            .dst_stage_mask(stage_mask)
            .dst_access_mask(access_mask);

        let subpass_descriptions = [subpass_description];
        let subpass_dependecies = [subpass_dependency];

        let create_info = vk::RenderPassCreateInfo::default()
            .attachments(&attachment_descriptions)
            .subpasses(&subpass_descriptions)
            .dependencies(&subpass_dependecies);

        let handle = unsafe { Context::get_device().create_render_pass(&create_info, None) }
            .expect("Failed to create render pass");

        RenderPass {
            handle,
            attachments: self.attachments.clone(),
        }
    }
}
