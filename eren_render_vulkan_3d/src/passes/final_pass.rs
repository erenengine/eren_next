use ash::vk;
use eren_render_vulkan_core::renderer::FrameContext;
use thiserror::Error;

use crate::{constants::CLEAR_COLOR, shader::create_shader_module};

const VERT_SHADER_BYTES: &[u8] = include_bytes!("../shaders/final.vert.spv");
const FRAG_SHADER_BYTES: &[u8] = include_bytes!("../shaders/final.frag.spv");

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
pub enum FinalPassError {
    #[error("Failed to create render pass: {0}")]
    RenderPassCreationFailed(String),

    #[error("Failed to create framebuffer: {0}")]
    FramebufferCreationFailed(String),

    #[error("Failed to create pipeline layout: {0}")]
    PipelineLayoutCreationFailed(String),

    #[error("Failed to create shader module: {0}")]
    ShaderModuleCreationFailed(String),

    #[error("Failed to create pipeline: {0}")]
    PipelineCreationFailed(String),
}

pub struct FinalPass {
    device: ash::Device,
    render_pass: vk::RenderPass,
    swapchain_framebuffers: Vec<vk::Framebuffer>,
    render_area: vk::Rect2D,

    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
}

impl FinalPass {
    pub fn new(
        device: ash::Device,
        swapchain_image_views: &Vec<vk::ImageView>,
        surface_format: vk::Format,
        image_extent: vk::Extent2D,
        descriptor_set_layout: vk::DescriptorSetLayout,
    ) -> Result<Self, FinalPassError> {
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

        let subpass = vk::SubpassDescription2::default()
            .color_attachments(std::slice::from_ref(&color_attachment_ref))
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS);

        let subpass_dependency = vk::SubpassDependency2::default()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_subpass(0)
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);

        let render_pass_info = vk::RenderPassCreateInfo2::default()
            .attachments(std::slice::from_ref(&color_attachment))
            .subpasses(std::slice::from_ref(&subpass))
            .dependencies(std::slice::from_ref(&subpass_dependency));

        let render_pass = unsafe {
            device
                .create_render_pass2(&render_pass_info, None)
                .map_err(|e| FinalPassError::RenderPassCreationFailed(e.to_string()))?
        };

        let mut swapchain_framebuffers = Vec::new();
        for &view in swapchain_image_views.iter() {
            let framebuffer_info = vk::FramebufferCreateInfo::default()
                .render_pass(render_pass)
                .attachments(std::slice::from_ref(&view))
                .width(image_extent.width)
                .height(image_extent.height)
                .layers(1);
            let framebuffer = unsafe {
                device
                    .create_framebuffer(&framebuffer_info, None)
                    .map_err(|e| FinalPassError::FramebufferCreationFailed(e.to_string()))?
            };
            swapchain_framebuffers.push(framebuffer);
        }

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(std::slice::from_ref(&descriptor_set_layout));

        let pipeline_layout = unsafe {
            device
                .create_pipeline_layout(&pipeline_layout_info, None)
                .map_err(|e| FinalPassError::PipelineLayoutCreationFailed(e.to_string()))?
        };

        let vertex_shader_module = create_shader_module(&device, VERT_SHADER_BYTES)
            .map_err(|e| FinalPassError::ShaderModuleCreationFailed(e.to_string()))?;

        let fragment_shader_module = create_shader_module(&device, FRAG_SHADER_BYTES)
            .map_err(|e| FinalPassError::ShaderModuleCreationFailed(e.to_string()))?;

        let main_function_name = std::ffi::CString::new("main").unwrap();

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vertex_shader_module)
                .name(&main_function_name),
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(fragment_shader_module)
                .name(&main_function_name),
        ];

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default();

        let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

        let viewport = vk::Viewport {
            x: 0.,
            y: 0.,
            width: image_extent.width as f32,
            height: image_extent.height as f32,
            min_depth: 0.,
            max_depth: 1.,
        };

        let scissors = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: image_extent,
        };

        let viewport_info = vk::PipelineViewportStateCreateInfo::default()
            .viewports(std::slice::from_ref(&viewport))
            .scissors(std::slice::from_ref(&scissors));

        let rasterizer_info = vk::PipelineRasterizationStateCreateInfo::default()
            .line_width(1.0)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .cull_mode(vk::CullModeFlags::NONE)
            .polygon_mode(vk::PolygonMode::FILL);

        let multisampler_info = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .alpha_blend_op(vk::BlendOp::ADD)
            .color_write_mask(
                vk::ColorComponentFlags::R
                    | vk::ColorComponentFlags::G
                    | vk::ColorComponentFlags::B
                    | vk::ColorComponentFlags::A,
            );

        let color_blend_info = vk::PipelineColorBlendStateCreateInfo::default()
            .attachments(std::slice::from_ref(&color_blend_attachment));

        let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly_info)
            .viewport_state(&viewport_info)
            .rasterization_state(&rasterizer_info)
            .multisample_state(&multisampler_info)
            .color_blend_state(&color_blend_info)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0);

        let pipeline = unsafe {
            device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
                .map_err(|e| FinalPassError::PipelineCreationFailed(e.1.to_string()))?
        }[0];

        unsafe {
            device.destroy_shader_module(vertex_shader_module, None);
            device.destroy_shader_module(fragment_shader_module, None);
        }

        Ok(Self {
            device,
            render_pass,
            swapchain_framebuffers,
            render_area: vk::Rect2D::default()
                .offset(vk::Offset2D::default())
                .extent(image_extent),

            pipeline,
            pipeline_layout,
        })
    }

    pub fn record(&self, frame_context: &FrameContext, descriptor_set: vk::DescriptorSet) {
        let render_pass_begin_info = vk::RenderPassBeginInfo::default()
            .render_pass(self.render_pass)
            .framebuffer(self.swapchain_framebuffers[frame_context.image_index])
            .render_area(self.render_area)
            .clear_values(&CLEAR_VALUES);

        let subpass_begin_info =
            vk::SubpassBeginInfo::default().contents(vk::SubpassContents::INLINE);

        unsafe {
            self.device.cmd_begin_render_pass2(
                frame_context.command_buffer,
                &render_pass_begin_info,
                &subpass_begin_info,
            );

            self.device.cmd_bind_pipeline(
                frame_context.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
            );

            self.device.cmd_bind_descriptor_sets(
                frame_context.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout,
                0,
                &[descriptor_set],
                &[],
            );

            self.device
                .cmd_draw(frame_context.command_buffer, 3, 1, 0, 0);

            self.device
                .cmd_end_render_pass2(frame_context.command_buffer, &vk::SubpassEndInfo::default());
        }
    }
}

impl Drop for FinalPass {
    fn drop(&mut self) {
        unsafe {
            self.device
                .device_wait_idle()
                .expect("Failed to wait for device idle");

            self.device.destroy_pipeline(self.pipeline, None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);

            for &framebuffer in self.swapchain_framebuffers.iter() {
                self.device.destroy_framebuffer(framebuffer, None);
            }

            self.device.destroy_render_pass(self.render_pass, None);
        }
    }
}
