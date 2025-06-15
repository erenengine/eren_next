use ash::vk;
use eren_render_vulkan_core::{
    renderer::FrameContext,
    vulkan::memory::{MemoryError, create_buffer_with_memory, create_image_with_memory},
};
use thiserror::Error;

use crate::{
    constants::CLEAR_COLOR, render::render_item::RenderItem, shader::create_shader_module,
};

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

pub struct CameraUBO {
    pub view_proj: glam::Mat4,
    pub light_view_proj: glam::Mat4,
    pub light_dir: glam::Vec3,
    // Alignment padding to satisfy std140 layout
    pub _pad: f32,
}

#[derive(Debug, Error)]
pub enum GeometryPassError {
    #[error("Failed to create image: {0}")]
    CreateImageFailed(MemoryError),

    #[error("Failed to create image view: {0}")]
    CreateImageViewFailed(String),

    #[error("Failed to create buffer: {0}")]
    CreateBufferFailed(MemoryError),

    #[error("Failed to create render pass: {0}")]
    RenderPassCreationFailed(String),

    #[error("Failed to create framebuffer: {0}")]
    FramebufferCreationFailed(String),

    #[error("Failed to create sampler: {0}")]
    SamplerCreationFailed(String),

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

    #[error("Failed to map memory: {0}")]
    MemoryMappingFailed(String),
}

pub struct GeometryPass {
    device: ash::Device,

    color_image: vk::Image,
    color_image_memory: vk::DeviceMemory,
    pub color_image_view: vk::ImageView,

    camera_buffer: vk::Buffer,
    camera_buffer_memory: vk::DeviceMemory,

    render_pass: vk::RenderPass,
    framebuffer: vk::Framebuffer,
    render_area: vk::Rect2D,

    shadow_sampler: vk::Sampler,
    descriptor_pool: vk::DescriptorPool,
    camera_descriptor_set_layout: vk::DescriptorSetLayout,
    camera_descriptor_set: vk::DescriptorSet,
    shadow_descriptor_set_layout: vk::DescriptorSetLayout,
    shadow_descriptor_set: vk::DescriptorSet,

    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
}

impl GeometryPass {
    pub fn new(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: ash::Device,
        image_extent: vk::Extent2D,
        shadow_depth_image_view: vk::ImageView,
    ) -> Result<Self, GeometryPassError> {
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

        let (color_image, color_image_memory) = create_image_with_memory(
            instance,
            physical_device,
            &device,
            &image_info,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )
        .map_err(|e| GeometryPassError::CreateImageFailed(e))?;

        let camera_buffer_size = std::mem::size_of::<CameraUBO>() as vk::DeviceSize;
        let (camera_buffer, camera_buffer_memory) = create_buffer_with_memory(
            instance,
            physical_device,
            &device,
            camera_buffer_size,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        )
        .map_err(|e| GeometryPassError::CreateBufferFailed(e))?;

        let image_view_info = vk::ImageViewCreateInfo::default()
            .image(color_image)
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

        let color_image_view: vk::ImageView = unsafe {
            device
                .create_image_view(&image_view_info, None)
                .map_err(|e| GeometryPassError::CreateImageViewFailed(e.to_string()))?
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
                .map_err(|e| GeometryPassError::RenderPassCreationFailed(e.to_string()))?
        };

        let framebuffer_info = vk::FramebufferCreateInfo::default()
            .render_pass(render_pass)
            .attachments(std::slice::from_ref(&color_image_view))
            .width(image_extent.width)
            .height(image_extent.height)
            .layers(1);

        let framebuffer = unsafe {
            device
                .create_framebuffer(&framebuffer_info, None)
                .map_err(|e| GeometryPassError::FramebufferCreationFailed(e.to_string()))?
        };

        let camera_descriptor_set_layout_binding = vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT);

        let camera_descriptor_set_layout_info = vk::DescriptorSetLayoutCreateInfo::default()
            .bindings(std::slice::from_ref(&camera_descriptor_set_layout_binding));

        let camera_descriptor_set_layout = unsafe {
            device
                .create_descriptor_set_layout(&camera_descriptor_set_layout_info, None)
                .map_err(|e| GeometryPassError::DescriptorSetLayoutCreationFailed(e.to_string()))?
        };

        let shadow_sampler_info = vk::SamplerCreateInfo {
            mag_filter: vk::Filter::LINEAR,
            min_filter: vk::Filter::LINEAR,
            mipmap_mode: vk::SamplerMipmapMode::NEAREST,
            address_mode_u: vk::SamplerAddressMode::CLAMP_TO_EDGE,
            address_mode_v: vk::SamplerAddressMode::CLAMP_TO_EDGE,
            address_mode_w: vk::SamplerAddressMode::CLAMP_TO_EDGE,
            border_color: vk::BorderColor::FLOAT_OPAQUE_WHITE,
            min_lod: 0.0,
            max_lod: 1.0,
            ..Default::default()
        };

        let shadow_sampler = unsafe {
            device
                .create_sampler(&shadow_sampler_info, None)
                .map_err(|e| GeometryPassError::SamplerCreationFailed(e.to_string()))?
        };

        let shadow_descriptor_set_layout_binding = vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT);

        let shadow_descriptor_set_layout_info = vk::DescriptorSetLayoutCreateInfo::default()
            .bindings(std::slice::from_ref(&shadow_descriptor_set_layout_binding));

        let shadow_descriptor_set_layout = unsafe {
            device
                .create_descriptor_set_layout(&shadow_descriptor_set_layout_info, None)
                .map_err(|e| GeometryPassError::DescriptorSetLayoutCreationFailed(e.to_string()))?
        };

        let pool_sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 1,
            },
        ];

        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .max_sets(2)
            .pool_sizes(&pool_sizes);

        let descriptor_pool = unsafe {
            device
                .create_descriptor_pool(&pool_info, None)
                .map_err(|e| GeometryPassError::DescriptorPoolCreationFailed(e.to_string()))?
        };

        let camera_alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(std::slice::from_ref(&camera_descriptor_set_layout));

        let camera_descriptor_set = unsafe {
            device
                .allocate_descriptor_sets(&camera_alloc_info)
                .map_err(|e| GeometryPassError::DescriptorSetAllocationFailed(e.to_string()))?[0]
        };

        let camera_buffer_info = vk::DescriptorBufferInfo::default()
            .buffer(camera_buffer)
            .offset(0)
            .range(camera_buffer_size);

        let camera_write = vk::WriteDescriptorSet::default()
            .dst_set(camera_descriptor_set)
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(std::slice::from_ref(&camera_buffer_info));

        unsafe {
            device.update_descriptor_sets(&[camera_write], &[]);
        }

        let shadow_alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(std::slice::from_ref(&shadow_descriptor_set_layout));

        let shadow_descriptor_set = unsafe {
            device
                .allocate_descriptor_sets(&shadow_alloc_info)
                .map_err(|e| GeometryPassError::DescriptorSetAllocationFailed(e.to_string()))?[0]
        };

        let shadow_image_info = vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL)
            .image_view(shadow_depth_image_view)
            .sampler(shadow_sampler);

        let shadow_write = vk::WriteDescriptorSet::default()
            .dst_set(shadow_descriptor_set)
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(std::slice::from_ref(&shadow_image_info));

        unsafe {
            device.update_descriptor_sets(&[shadow_write], &[]);
        }

        let set_layouts = [camera_descriptor_set_layout, shadow_descriptor_set_layout];

        // Pipeline layout with descriptor set + push constant
        let push_constant_range = vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .offset(0)
            .size(64); // mat4

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(&set_layouts)
            .push_constant_ranges(std::slice::from_ref(&push_constant_range));

        let pipeline_layout = unsafe {
            device
                .create_pipeline_layout(&pipeline_layout_info, None)
                .map_err(|e| GeometryPassError::PipelineLayoutCreationFailed(e.to_string()))?
        };

        let vertex_shader_module = create_shader_module(&device, VERT_SHADER_BYTES)
            .map_err(|e| GeometryPassError::ShaderModuleCreationFailed(e.to_string()))?;

        let fragment_shader_module = create_shader_module(&device, FRAG_SHADER_BYTES)
            .map_err(|e| GeometryPassError::ShaderModuleCreationFailed(e.to_string()))?;

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

        let binding_description = vk::VertexInputBindingDescription {
            binding: 0,
            stride: std::mem::size_of::<[f32; 8]>() as u32,
            input_rate: vk::VertexInputRate::VERTEX,
        };

        let attribute_descriptions = [
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: 0,
            },
            vk::VertexInputAttributeDescription {
                location: 1,
                binding: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: 12, // 3 * 4 bytes
            },
            vk::VertexInputAttributeDescription {
                location: 2,
                binding: 0,
                format: vk::Format::R32G32_SFLOAT,
                offset: 24, // 3 * 4 + 3 * 4 bytes
            },
        ];

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(std::slice::from_ref(&binding_description))
            .vertex_attribute_descriptions(&attribute_descriptions);

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
                .map_err(|e| GeometryPassError::PipelineCreationFailed(e.1.to_string()))?
        }[0];

        unsafe {
            device.destroy_shader_module(vertex_shader_module, None);
            device.destroy_shader_module(fragment_shader_module, None);
        }

        Ok(Self {
            device,

            color_image,
            color_image_memory,
            color_image_view,

            camera_buffer,
            camera_buffer_memory,

            render_pass,
            framebuffer,
            render_area: vk::Rect2D::default()
                .offset(vk::Offset2D::default())
                .extent(image_extent),

            shadow_sampler,
            descriptor_pool,
            camera_descriptor_set_layout,
            camera_descriptor_set,
            shadow_descriptor_set_layout,
            shadow_descriptor_set,

            pipeline_layout,
            pipeline,
        })
    }

    pub fn upload_camera_buffer(&self, camera: &CameraUBO) -> Result<(), GeometryPassError> {
        unsafe {
            self.device
                .map_memory(
                    self.camera_buffer_memory,
                    0,
                    std::mem::size_of::<CameraUBO>() as vk::DeviceSize,
                    vk::MemoryMapFlags::empty(),
                )
                .map_err(|e| GeometryPassError::MemoryMappingFailed(e.to_string()))
                .and_then(|ptr| {
                    std::ptr::copy_nonoverlapping(camera, ptr as *mut CameraUBO, 1);
                    Ok(())
                })
                .map_err(|e| GeometryPassError::MemoryMappingFailed(e.to_string()))?;
        }

        Ok(())
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
                &[self.camera_descriptor_set, self.shadow_descriptor_set],
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

impl Drop for GeometryPass {
    fn drop(&mut self) {
        unsafe {
            self.device
                .device_wait_idle()
                .expect("Failed to wait for device idle");

            self.device.destroy_pipeline(self.pipeline, None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);

            self.device.destroy_sampler(self.shadow_sampler, None);
            self.device
                .destroy_descriptor_pool(self.descriptor_pool, None);
            self.device
                .destroy_descriptor_set_layout(self.camera_descriptor_set_layout, None);
            self.device
                .destroy_descriptor_set_layout(self.shadow_descriptor_set_layout, None);

            self.device.destroy_framebuffer(self.framebuffer, None);
            self.device.destroy_render_pass(self.render_pass, None);

            self.device.destroy_buffer(self.camera_buffer, None);
            self.device.free_memory(self.camera_buffer_memory, None);

            self.device.destroy_image_view(self.color_image_view, None);
            self.device.destroy_image(self.color_image, None);
            self.device.free_memory(self.color_image_memory, None);
        }
    }
}
