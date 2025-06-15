use ash::vk;
use eren_render_vulkan_core::{
    renderer::FrameContext,
    vulkan::memory::{MemoryError, create_buffer_with_memory, create_image_with_memory},
};
use thiserror::Error;

use crate::{render::render_item::RenderItem, shader::create_shader_module};

const VERT_SHADER_BYTES: &[u8] = include_bytes!("../shaders/shadow.vert.spv");

const CLEAR_VALUES: [vk::ClearValue; 1] = [vk::ClearValue {
    depth_stencil: vk::ClearDepthStencilValue {
        depth: 1.0,
        stencil: 0,
    },
}];

pub struct LightVP {
    pub light_view_proj: [[f32; 4]; 4],
}

#[derive(Debug, Error)]
pub enum ShadowPassError {
    #[error("Failed to create image: {0}")]
    CreateImageFailed(MemoryError),

    #[error("Failed to create image view: {0}")]
    CreateImageViewFailed(String),

    #[error("Failed to create render pass: {0}")]
    CreateRenderPassFailed(String),

    #[error("Failed to create framebuffer: {0}")]
    FramebufferCreationFailed(String),

    #[error("Failed to create buffer: {0}")]
    CreateBufferFailed(MemoryError),

    #[error("Failed to create descriptor set layout: {0}")]
    DescriptorSetLayoutCreationFailed(String),

    #[error("Failed to create descriptor pool: {0}")]
    DescriptorPoolCreationFailed(String),

    #[error("Failed to allocate descriptor set: {0}")]
    DescriptorSetAllocationFailed(String),

    #[error("Failed to create pipeline layout: {0}")]
    PipelineLayoutCreationFailed(String),

    #[error("Failed to create shader module: {0}")]
    ShaderModuleCreationFailed(String),

    #[error("Failed to create pipeline: {0}")]
    PipelineCreationFailed(String),
}

pub struct ShadowPass {
    device: ash::Device,

    depth_image: vk::Image,
    depth_image_memory: vk::DeviceMemory,
    pub depth_image_view: vk::ImageView,

    light_vp_buffer: vk::Buffer,
    light_vp_buffer_memory: vk::DeviceMemory,

    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layout: vk::DescriptorSetLayout,
    descriptor_set: vk::DescriptorSet,

    render_pass: vk::RenderPass,
    framebuffer: vk::Framebuffer,
    render_area: vk::Rect2D,

    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
}

impl ShadowPass {
    pub fn new(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: ash::Device,
        image_extent: vk::Extent2D,
    ) -> Result<Self, ShadowPassError> {
        let depth_format = vk::Format::D32_SFLOAT;

        let image_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .format(depth_format)
            .extent(vk::Extent3D {
                width: image_extent.width,
                height: image_extent.height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | vk::ImageUsageFlags::SAMPLED)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::UNDEFINED);

        let (depth_image, depth_image_memory) = create_image_with_memory(
            instance,
            physical_device,
            &device,
            &image_info,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )
        .map_err(|e| ShadowPassError::CreateImageFailed(e))?;

        let depth_image_view_info = vk::ImageViewCreateInfo::default()
            .image(depth_image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(depth_format)
            .subresource_range(
                vk::ImageSubresourceRange::default()
                    .aspect_mask(vk::ImageAspectFlags::DEPTH)
                    .base_mip_level(0)
                    .level_count(1)
                    .base_array_layer(0)
                    .layer_count(1),
            );

        let depth_image_view: vk::ImageView = unsafe {
            device
                .create_image_view(&depth_image_view_info, None)
                .map_err(|e| ShadowPassError::CreateImageViewFailed(e.to_string()))?
        };

        let depth_attachment = vk::AttachmentDescription2::default()
            .format(depth_format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL);

        let depth_attachment_ref = vk::AttachmentReference2::default()
            .attachment(0)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .aspect_mask(vk::ImageAspectFlags::DEPTH);

        let subpass = vk::SubpassDescription2::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .depth_stencil_attachment(&depth_attachment_ref);

        let render_pass_info = vk::RenderPassCreateInfo2::default()
            .attachments(std::slice::from_ref(&depth_attachment))
            .subpasses(std::slice::from_ref(&subpass));

        let render_pass = unsafe {
            device
                .create_render_pass2(&render_pass_info, None)
                .map_err(|e| ShadowPassError::CreateRenderPassFailed(e.to_string()))?
        };

        let framebuffer_info = vk::FramebufferCreateInfo::default()
            .render_pass(render_pass)
            .attachments(std::slice::from_ref(&depth_image_view))
            .width(image_extent.width)
            .height(image_extent.height)
            .layers(1);

        let framebuffer = unsafe {
            device
                .create_framebuffer(&framebuffer_info, None)
                .map_err(|e| ShadowPassError::FramebufferCreationFailed(e.to_string()))?
        };

        let light_vp_buffer_size = std::mem::size_of::<LightVP>() as vk::DeviceSize;
        let (light_vp_buffer, light_vp_buffer_memory) = create_buffer_with_memory(
            instance,
            physical_device,
            &device,
            light_vp_buffer_size,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )
        .map_err(|e| ShadowPassError::CreateBufferFailed(e))?;

        // Descriptor Set Layout
        let ubo_layout_binding = vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX);

        let layout_info = vk::DescriptorSetLayoutCreateInfo::default()
            .bindings(std::slice::from_ref(&ubo_layout_binding));

        let descriptor_set_layout = unsafe {
            device
                .create_descriptor_set_layout(&layout_info, None)
                .map_err(|e| ShadowPassError::DescriptorSetLayoutCreationFailed(e.to_string()))?
        };

        // Descriptor Pool
        let pool_size = vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
        };

        let descriptor_pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(std::slice::from_ref(&pool_size))
            .max_sets(1);

        let descriptor_pool = unsafe {
            device
                .create_descriptor_pool(&descriptor_pool_info, None)
                .map_err(|e| ShadowPassError::DescriptorPoolCreationFailed(e.to_string()))?
        };

        // Descriptor Set Allocation
        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(std::slice::from_ref(&descriptor_set_layout));

        let descriptor_set = unsafe {
            device
                .allocate_descriptor_sets(&alloc_info)
                .map_err(|e| ShadowPassError::DescriptorSetAllocationFailed(e.to_string()))?[0]
        };

        // Descriptor Write
        let buffer_info = vk::DescriptorBufferInfo::default()
            .buffer(light_vp_buffer)
            .offset(0)
            .range(light_vp_buffer_size);

        let write = vk::WriteDescriptorSet::default()
            .dst_set(descriptor_set)
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(std::slice::from_ref(&buffer_info));

        unsafe {
            device.update_descriptor_sets(&[write], &[]);
        }

        // Pipeline layout with descriptor set + push constant
        let push_constant_range = vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .offset(0)
            .size(64); // mat4

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(std::slice::from_ref(&descriptor_set_layout))
            .push_constant_ranges(std::slice::from_ref(&push_constant_range));

        let pipeline_layout = unsafe {
            device
                .create_pipeline_layout(&pipeline_layout_info, None)
                .map_err(|e| ShadowPassError::PipelineLayoutCreationFailed(e.to_string()))?
        };

        let vertex_shader_module = create_shader_module(&device, VERT_SHADER_BYTES)
            .map_err(|e| ShadowPassError::ShaderModuleCreationFailed(e.to_string()))?;

        let main_function_name = std::ffi::CString::new("main").unwrap();

        let shader_stage = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vertex_shader_module)
            .name(&main_function_name);

        let binding_description = vk::VertexInputBindingDescription::default()
            .binding(0)
            .stride(std::mem::size_of::<[f32; 3]>() as u32) // vec3 position
            .input_rate(vk::VertexInputRate::VERTEX);

        let attribute_description = vk::VertexInputAttributeDescription::default()
            .location(0)
            .binding(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(0);

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(std::slice::from_ref(&binding_description))
            .vertex_attribute_descriptions(std::slice::from_ref(&attribute_description));

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
            .cull_mode(vk::CullModeFlags::BACK)
            .polygon_mode(vk::PolygonMode::FILL);

        let depth_stencil_info = vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL);

        let multisampler_info = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(std::slice::from_ref(&shader_stage))
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly_info)
            .viewport_state(&viewport_info)
            .rasterization_state(&rasterizer_info)
            .multisample_state(&multisampler_info)
            .depth_stencil_state(&depth_stencil_info)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0);

        let pipeline = unsafe {
            device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
                .map_err(|e| ShadowPassError::PipelineCreationFailed(e.1.to_string()))?
        }[0];

        unsafe { device.destroy_shader_module(vertex_shader_module, None) };

        Ok(Self {
            device,

            depth_image,
            depth_image_memory,
            depth_image_view,

            light_vp_buffer,
            light_vp_buffer_memory,

            descriptor_pool,
            descriptor_set_layout,
            descriptor_set,

            render_pass,
            framebuffer,
            render_area: vk::Rect2D::default()
                .offset(vk::Offset2D::default())
                .extent(image_extent),

            pipeline_layout,
            pipeline,
        })
    }

    pub fn record(&self, frame_context: &FrameContext, render_items: &[RenderItem]) {
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

            self.device.cmd_bind_descriptor_sets(
                frame_context.command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout,
                0,
                &[self.descriptor_set],
                &[],
            );

            for render_item in render_items {
                self.device.cmd_bind_vertex_buffers(
                    frame_context.command_buffer,
                    0,
                    &[render_item.mesh.vertex_buffer],
                    &[0],
                );

                self.device.cmd_bind_index_buffer(
                    frame_context.command_buffer,
                    render_item.mesh.index_buffer,
                    0,
                    vk::IndexType::UINT32,
                );

                let mat_ref: &[f32; 16] = std::mem::transmute(&render_item.transform);
                let bytes: &[u8] = std::slice::from_raw_parts(
                    mat_ref.as_ptr() as *const u8,
                    std::mem::size_of::<[f32; 16]>(),
                );

                self.device.cmd_push_constants(
                    frame_context.command_buffer,
                    self.pipeline_layout,
                    vk::ShaderStageFlags::VERTEX,
                    0,
                    bytes,
                );

                self.device.cmd_draw_indexed(
                    frame_context.command_buffer,
                    render_item.mesh.index_count,
                    1,
                    0,
                    0,
                    0,
                );
            }

            self.device
                .cmd_end_render_pass2(frame_context.command_buffer, &vk::SubpassEndInfo::default());
        }
    }
}

impl Drop for ShadowPass {
    fn drop(&mut self) {
        unsafe {
            self.device
                .device_wait_idle()
                .expect("Failed to wait for device idle");

            self.device.destroy_pipeline(self.pipeline, None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);

            self.device
                .destroy_descriptor_pool(self.descriptor_pool, None);
            self.device
                .destroy_descriptor_set_layout(self.descriptor_set_layout, None);

            self.device.destroy_buffer(self.light_vp_buffer, None);
            self.device.free_memory(self.light_vp_buffer_memory, None);

            self.device.destroy_framebuffer(self.framebuffer, None);
            self.device.destroy_render_pass(self.render_pass, None);

            self.device.destroy_image_view(self.depth_image_view, None);
            self.device.destroy_image(self.depth_image, None);
            self.device.free_memory(self.depth_image_memory, None);
        }
    }
}
