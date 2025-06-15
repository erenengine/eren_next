use ash::vk;
use thiserror::Error;

use crate::constants::CLEAR_COLOR;

const VERT_SHADER_BYTES: &[u8] = include_bytes!("../shaders/geometry.vert.spv");
const FRAG_SHADER_BYTES: &[u8] = include_bytes!("../shaders/geometry.frag.spv");

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
pub enum GeometryPassError {
    #[error("Failed to create render pass: {0}")]
    RenderPassCreationFailed(String),
}

pub struct GeometryPass {
    device: ash::Device,
    render_pass: vk::RenderPass,
}

impl GeometryPass {
    pub fn new(
        device: ash::Device,
        surface_format: vk::Format,
        image_extent: vk::Extent2D,
        shadow_depth_image_view: vk::ImageView,
    ) -> Result<Self, GeometryPassError> {
        let color_attachment = vk::AttachmentDescription2::default()
            .format(surface_format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

        let depth_attachment = vk::AttachmentDescription2::default()
            .format(vk::Format::D32_SFLOAT)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);

        let color_attachment_ref = vk::AttachmentReference2::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .aspect_mask(vk::ImageAspectFlags::COLOR);

        let depth_attachment_ref = vk::AttachmentReference2::default()
            .attachment(1)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .aspect_mask(vk::ImageAspectFlags::DEPTH);

        let subpass = vk::SubpassDescription2::default()
            .color_attachments(std::slice::from_ref(&color_attachment_ref))
            .depth_stencil_attachment(&depth_attachment_ref)
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS);

        let attachments = [color_attachment, depth_attachment];

        let render_pass_info = vk::RenderPassCreateInfo2::default()
            .attachments(&attachments)
            .subpasses(std::slice::from_ref(&subpass));

        let render_pass = unsafe {
            device
                .create_render_pass2(&render_pass_info, None)
                .map_err(|e| GeometryPassError::RenderPassCreationFailed(e.to_string()))?
        };

        let framebuffer_info = vk::FramebufferCreateInfo::default()
            .render_pass(render_pass)
            .attachments(&[color_attachment_view, depth_attachment_view])
            .width(extent.width)
            .height(extent.height)
            .layers(1);

        let framebuffer = unsafe {
            device
                .create_framebuffer(&framebuffer_info, None)
                .expect("Failed to create framebuffer")
        };

        Ok(Self {
            device,
            render_pass,
        })
    }

    pub fn record(
        &self,
        framebuffer: vk::Framebuffer,
        render_area: vk::Rect2D,
        command_buffer: vk::CommandBuffer,
    ) {
        let render_pass_begin_info = vk::RenderPassBeginInfo::default()
            .render_pass(self.render_pass)
            .framebuffer(framebuffer)
            .render_area(render_area)
            .clear_values(&CLEAR_VALUES);

        let subpass_begin_info =
            vk::SubpassBeginInfo::default().contents(vk::SubpassContents::INLINE);

        unsafe {
            self.device.cmd_begin_render_pass2(
                command_buffer,
                &render_pass_begin_info,
                &subpass_begin_info,
            );

            //TODO:
            println!("Recording geometry pass");

            self.device
                .cmd_end_render_pass2(command_buffer, &vk::SubpassEndInfo::default());
        }
    }
}

impl<'a> Drop for GeometryPass<'a> {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_render_pass(self.render_pass, None);
        }
    }
}
