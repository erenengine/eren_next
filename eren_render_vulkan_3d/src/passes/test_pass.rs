use ash::vk;
use eren_render_vulkan_core::{
    renderer::FrameContext,
    vulkan::memory::{MemoryError, create_image_with_memory},
};
use thiserror::Error;

use crate::{constants::CLEAR_COLOR, shader::create_shader_module};

const VERT_SHADER_BYTES: &[u8] = include_bytes!("../shaders/test.vert.spv");
const FRAG_SHADER_BYTES: &[u8] = include_bytes!("../shaders/test.frag.spv");

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
    #[error("Failed to create image: {0}")]
    CreateImageFailed(MemoryError),

    #[error("Failed to create image view: {0}")]
    CreateImageViewFailed(String),

    #[error("Failed to create render pass: {0}")]
    RenderPassCreationFailed(String),

    #[error("Failed to create framebuffer: {0}")]
    FramebufferCreationFailed(String),

    #[error("Failed to create descriptor set layout: {0}")]
    DescriptorSetLayoutCreationFailed(String),

    #[error("Failed to create pipeline layout: {0}")]
    PipelineLayoutCreationFailed(String),

    #[error("Failed to create shader module: {0}")]
    ShaderModuleCreationFailed(String),

    #[error("Failed to create pipeline: {0}")]
    PipelineCreationFailed(String),
}

pub struct TestPass {
    device: ash::Device,

    render_pass: vk::RenderPass,
    framebuffer: vk::Framebuffer,
    render_area: vk::Rect2D,

    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,

    image: vk::Image,
    image_memory: vk::DeviceMemory,
    image_view: vk::ImageView,
}

impl TestPass {
    pub fn new(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: ash::Device,
        image_extent: vk::Extent2D,
    ) -> Result<Self, TestPassError> {
        let color_format = vk::Format::R8G8B8A8_UNORM;

        let image_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .format(color_format)
            .extent(vk::Extent3D {
                width: image_extent.width,
                height: image_extent.height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED)
            .initial_layout(vk::ImageLayout::UNDEFINED);

        let (image, image_memory) = create_image_with_memory(
            instance,
            physical_device,
            &device,
            &image_info,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )
        .map_err(|e| TestPassError::CreateImageFailed(e))?;

        let image_view_info = vk::ImageViewCreateInfo::default()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(color_format)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1),
            );

        let image_view: vk::ImageView = unsafe {
            device
                .create_image_view(&image_view_info, None)
                .map_err(|e| TestPassError::CreateImageViewFailed(e.to_string()))?
        };

        let color_attachment = vk::AttachmentDescription2::default()
            .format(color_format)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .samples(vk::SampleCountFlags::TYPE_1);

        let color_attachment_ref = vk::AttachmentReference2::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .aspect_mask(vk::ImageAspectFlags::COLOR);

        let subpass = vk::SubpassDescription2::default()
            .color_attachments(std::slice::from_ref(&color_attachment_ref))
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS);

        let render_pass_info = vk::RenderPassCreateInfo2::default()
            .attachments(std::slice::from_ref(&color_attachment))
            .subpasses(std::slice::from_ref(&subpass));

        let render_pass = unsafe {
            device
                .create_render_pass2(&render_pass_info, None)
                .map_err(|e| TestPassError::RenderPassCreationFailed(e.to_string()))?
        };

        let framebuffer_info = vk::FramebufferCreateInfo::default()
            .render_pass(render_pass)
            .attachments(std::slice::from_ref(&image_view))
            .width(image_extent.width)
            .height(image_extent.height)
            .layers(1);

        let framebuffer = unsafe {
            device
                .create_framebuffer(&framebuffer_info, None)
                .map_err(|e| TestPassError::FramebufferCreationFailed(e.to_string()))?
        };

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default();

        let pipeline_layout = unsafe {
            device
                .create_pipeline_layout(&pipeline_layout_info, None)
                .map_err(|e| TestPassError::PipelineLayoutCreationFailed(e.to_string()))?
        };

        let vertex_shader_module = create_shader_module(&device, VERT_SHADER_BYTES)
            .map_err(|e| TestPassError::ShaderModuleCreationFailed(e.to_string()))?;

        let fragment_shader_module = create_shader_module(&device, FRAG_SHADER_BYTES)
            .map_err(|e| TestPassError::ShaderModuleCreationFailed(e.to_string()))?;

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
            .topology(vk::PrimitiveTopology::POINT_LIST);

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
                .map_err(|e| TestPassError::PipelineCreationFailed(e.1.to_string()))?
        }[0];

        unsafe {
            device.destroy_shader_module(vertex_shader_module, None);
            device.destroy_shader_module(fragment_shader_module, None);
        }

        Ok(Self {
            device,

            render_pass,
            framebuffer,
            render_area: vk::Rect2D::default()
                .offset(vk::Offset2D::default())
                .extent(image_extent),

            pipeline,
            pipeline_layout,

            image,
            image_memory,
            image_view,
        })
    }

    pub fn record(&self, frame_context: &FrameContext) {
        let render_pass_begin_info = vk::RenderPassBeginInfo::default()
            .render_pass(self.render_pass)
            .framebuffer(self.framebuffer)
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

            self.device
                .cmd_draw(frame_context.command_buffer, 1, 1, 0, 0);

            self.device
                .cmd_end_render_pass2(frame_context.command_buffer, &vk::SubpassEndInfo::default());
        }
    }

    pub fn create_descriptor_set(
        &self,
        descriptor_pool: vk::DescriptorPool,
        descriptor_set_layout: vk::DescriptorSetLayout,
        sampler: vk::Sampler,
    ) -> Result<vk::DescriptorSet, TestPassError> {
        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(std::slice::from_ref(&descriptor_set_layout));

        let descriptor_set = unsafe {
            self.device
                .allocate_descriptor_sets(&alloc_info)
                .map_err(|e| TestPassError::DescriptorSetLayoutCreationFailed(e.to_string()))?[0]
        };

        let image_info = vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .image_view(self.image_view)
            .sampler(sampler);

        let write = vk::WriteDescriptorSet::default()
            .dst_set(descriptor_set)
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(std::slice::from_ref(&image_info));

        unsafe {
            self.device.update_descriptor_sets(&[write], &[]);
        }

        Ok(descriptor_set)
    }
}

impl Drop for TestPass {
    fn drop(&mut self) {
        unsafe {
            self.device
                .device_wait_idle()
                .expect("Failed to wait for device idle");

            self.device.destroy_pipeline(self.pipeline, None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);

            self.device.destroy_framebuffer(self.framebuffer, None);
            self.device.destroy_render_pass(self.render_pass, None);

            self.device.destroy_image_view(self.image_view, None);
            self.device.destroy_image(self.image, None);
            self.device.free_memory(self.image_memory, None);
        }
    }
}
