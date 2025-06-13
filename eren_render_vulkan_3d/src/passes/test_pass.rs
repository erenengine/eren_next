use std::sync::Arc;

use ash::vk;
use eren_render_vulkan_core::renderer::FrameContext;
use thiserror::Error;

use crate::constants::CLEAR_COLOR;

const CLEAR_VALUES: [vk::ClearValue; 2] = [
    vk::ClearValue {
        color: vk::ClearColorValue {
            float32: CLEAR_COLOR,
        },
    },
    vk::ClearValue {
        depth_stencil: vk::ClearDepthStencilValue {
            depth: 1.0,
            stencil: 0,
        },
    },
];

#[derive(Debug, Error)]
pub enum TestPassError {
    #[error("Failed to create render pass: {0}")]
    RenderPassCreationFailed(vk::Result),

    #[error("Failed to create framebuffer: {0}")]
    FramebufferCreationFailed(vk::Result),
}

pub struct TestPass {
    logical_device: Arc<ash::Device>,
    render_pass: vk::RenderPass,
    swapchain_framebuffers: Vec<vk::Framebuffer>,
    render_area: vk::Rect2D,
}

impl TestPass {
    pub fn new(
        logical_device: Arc<ash::Device>,
        swapchain_image_views: &Vec<vk::ImageView>,
        surface_format: vk::Format,
        image_extent: vk::Extent2D,
    ) -> Result<Self, TestPassError> {
        let color_attachment = vk::AttachmentDescription2::default()
            .format(surface_format)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .samples(vk::SampleCountFlags::TYPE_1);

        let color_attachment_ref = vk::AttachmentReference2::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .aspect_mask(vk::ImageAspectFlags::COLOR);

        let attachments = [color_attachment];

        let subpasses = [vk::SubpassDescription2::default()
            .color_attachments(std::slice::from_ref(&color_attachment_ref))
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)];

        let subpass_dependencies = [vk::SubpassDependency2::default()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_subpass(0)
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)];

        let create_render_pass_info = vk::RenderPassCreateInfo2::default()
            .attachments(&attachments)
            .subpasses(&subpasses)
            .dependencies(&subpass_dependencies);

        let render_pass = unsafe {
            logical_device
                .create_render_pass2(&create_render_pass_info, None)
                .map_err(|err| TestPassError::RenderPassCreationFailed(err))?
        };

        let mut swapchain_framebuffers = Vec::new();
        for &view in swapchain_image_views.iter() {
            let attachments = [view];
            let framebuffer_info = vk::FramebufferCreateInfo::default()
                .render_pass(render_pass)
                .attachments(&attachments)
                .width(image_extent.width)
                .height(image_extent.height)
                .layers(1);
            let framebuffer = unsafe {
                logical_device
                    .create_framebuffer(&framebuffer_info, None)
                    .map_err(|err| TestPassError::FramebufferCreationFailed(err))?
            };
            swapchain_framebuffers.push(framebuffer);
        }

        Ok(Self {
            logical_device,
            render_pass,
            swapchain_framebuffers,
            render_area: vk::Rect2D::default()
                .offset(vk::Offset2D::default())
                .extent(image_extent),
        })
    }

    pub fn record(&self, frame_context: &FrameContext) {
        let render_pass_begin_info = vk::RenderPassBeginInfo::default()
            .render_pass(self.render_pass)
            .framebuffer(self.swapchain_framebuffers[frame_context.image_index])
            .render_area(self.render_area)
            .clear_values(&CLEAR_VALUES);

        let subpass_begin_info =
            vk::SubpassBeginInfo::default().contents(vk::SubpassContents::INLINE);

        unsafe {
            self.logical_device.cmd_begin_render_pass2(
                frame_context.command_buffer,
                &render_pass_begin_info,
                &subpass_begin_info,
            );

            /*self.logical_device.cmd_bind_pipeline(
                frame_context.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.pipeline,
            );

            self.logical_device
                .cmd_draw(frame_context.command_buffer, 1, 1, 0, 0);*/

            self.logical_device
                .cmd_end_render_pass2(frame_context.command_buffer, &vk::SubpassEndInfo::default());
        }
    }
}

impl Drop for TestPass {
    fn drop(&mut self) {
        unsafe {
            self.logical_device
                .destroy_render_pass(self.render_pass, None);

            for &framebuffer in self.swapchain_framebuffers.iter() {
                self.logical_device.destroy_framebuffer(framebuffer, None);
            }
        }
    }
}
