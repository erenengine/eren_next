use ash::{Device, Instance as AshInstance, vk};
use eren_core::render_world::ash::buffer::{
    BufferResource, MemoryLocation, create_buffer_with_size,
};
// Aliased to avoid conflict with other Instance types
use glam::{Mat3, Vec2};
use std::{ffi::CStr, marker::PhantomData};
use winit::dpi::PhysicalSize;

const SPRITE_VERT_SHADER_BYTES: &[u8] = include_bytes!("sprite.vert.spv"); // Ensure path is correct
const SPRITE_FRAG_SHADER_BYTES: &[u8] = include_bytes!("sprite.frag.spv"); // Ensure path is correct

pub fn create_shader_module(device: &Device, code: &[u8]) -> Result<vk::ShaderModule, vk::Result> {
    let code_u32 = unsafe {
        std::slice::from_raw_parts(
            code.as_ptr() as *const u32,
            code.len() / std::mem::size_of::<u32>(),
        )
    };
    let create_info = vk::ShaderModuleCreateInfo::default().code(code_u32);
    unsafe { device.create_shader_module(&create_info, None) }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ScreenInfo {
    resolution: [f32; 2],
    scale_factor: f32,
    _padding: f32, // Ensure alignment for UBOs (often 16 bytes for vec3/mat3 members)
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    pos: [f32; 2],
    uv: [f32; 2],
}

const QUAD_VERTICES: [Vertex; 4] = [
    Vertex {
        pos: [-0.5, -0.5],
        uv: [0.0, 1.0],
    }, // bottom-left
    Vertex {
        pos: [0.5, -0.5],
        uv: [1.0, 1.0],
    }, // bottom-right
    Vertex {
        pos: [0.5, 0.5],
        uv: [1.0, 0.0],
    }, // top-right
    Vertex {
        pos: [-0.5, 0.5],
        uv: [0.0, 0.0],
    }, // top-left
];
const QUAD_INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceData {
    size: [f32; 2],
    // Mat3 is 3x3 floats. Vulkan std140 layout for mat3 is complex.
    // Often treated as 3 x vec4 where last component of vec4 is padding.
    // Or, pass as 3 vec3s if shader handles it, or ensure shader expects tight packing.
    // For simplicity, using [[f32;3];3] and hoping shader matches.
    // More robust: use mat4 and ignore last row/col or pass as array of vec4s.
    matrix_col0: [f32; 3], // Column 0
    matrix_col1: [f32; 3], // Column 1
    matrix_col2: [f32; 3], // Column 2
    alpha: f32,
    _padding_instance: [f32; 2], // Pad to align alpha, matrix is 9 floats, alpha 1 = 10. Next multiple of 4 is 12.
}

pub struct SpriteRenderCommand<SA> {
    pub size: Vec2,
    pub matrix: Mat3,
    pub alpha: f32,
    pub sprite_asset_id: SA, // For potential future use or debugging, not directly used in render if set is bound
    pub descriptor_set: vk::DescriptorSet,
}

pub struct AshSpriteRenderer<SA>
where
    SA: Copy + PartialEq,
{
    device: Option<Device>, // Use Option<Arc<Device>> if shared logic, or Option<Device>
    phys_mem_props: Option<vk::PhysicalDeviceMemoryProperties>, // For buffer creation

    // Pipeline and layout related
    pipeline_layout: Option<vk::PipelineLayout>,
    pipeline: Option<vk::Pipeline>,

    // Geometry buffers
    quad_vertex_buffer: Option<BufferResource>,
    quad_index_buffer: Option<BufferResource>,

    // Per-instance data
    instance_buffer: Option<BufferResource>,
    instance_capacity: usize, // Number of InstanceData elements

    // Screen Uniform Buffer
    screen_info_buffer: Option<BufferResource>,
    screen_descriptor_set_layout: Option<vk::DescriptorSetLayout>,
    screen_descriptor_pool: Option<vk::DescriptorPool>, // Own pool for screen UBO
    screen_descriptor_set: Option<vk::DescriptorSet>,

    current_window_size: PhysicalSize<u32>,
    current_scale_factor: f64,

    phantom: PhantomData<SA>,
}

impl<SA: Copy + PartialEq> AshSpriteRenderer<SA> {
    pub fn new() -> Self {
        Self {
            device: None,
            phys_mem_props: None,
            pipeline_layout: None,
            pipeline: None,
            quad_vertex_buffer: None,
            quad_index_buffer: None,
            instance_buffer: None,
            instance_capacity: 0,
            screen_info_buffer: None,
            screen_descriptor_set_layout: None,
            screen_descriptor_pool: None,
            screen_descriptor_set: None,
            current_window_size: PhysicalSize {
                width: 0,
                height: 0,
            },
            current_scale_factor: 1.0,
            phantom: PhantomData,
        }
    }

    pub fn on_gpu_resources_ready(
        &mut self,
        _instance_ash: &AshInstance, // Renamed to avoid conflict, _ if not used
        physical_device: vk::PhysicalDevice,
        device: Device, // Take ownership or Arc
        phys_mem_props: vk::PhysicalDeviceMemoryProperties,
        render_pass: vk::RenderPass, // Provided by GpuResourceManager
        sprite_texture_set_layout: vk::DescriptorSetLayout, // Provided by AssetManager
        window_size: PhysicalSize<u32>,
        scale_factor: f64,
        // max_sprites: u32, // This determines instance_capacity if dynamic, or initial capacity
    ) {
        if self.device.is_some() {
            self.on_gpu_resources_lost_internal(false); // Clean up before reinitializing
        }
        self.current_window_size = window_size;
        self.current_scale_factor = scale_factor;

        // --- Screen UBO and Descriptor Set (Set 0) ---
        let screen_info_data = ScreenInfo {
            resolution: [window_size.width as f32, window_size.height as f32],
            scale_factor: scale_factor as f32,
            _padding: 0.0,
        };
        let screen_ubo_buffer = create_buffer_with_size(
            &device,
            &phys_mem_props,
            std::mem::size_of::<ScreenInfo>() as vk::DeviceSize,
            Some(std::slice::from_ref(&screen_info_data)),
            vk::BufferUsageFlags::UNIFORM_BUFFER, // Not TRANSFER_DST if initialized directly and mapped
            MemoryLocation::CpuToGpu,             // Persistently mapped for updates
        );

        let screen_dsl_binding = vk::DescriptorSetLayoutBinding::default()
            .binding(0) // UBO binding
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX);
        let screen_dsl_info = vk::DescriptorSetLayoutCreateInfo::default()
            .bindings(std::slice::from_ref(&screen_dsl_binding));
        let screen_dsl = unsafe { device.create_descriptor_set_layout(&screen_dsl_info, None) }
            .expect("Failed to create screen descriptor set layout");

        let screen_pool_size = vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
        };
        let screen_pool_info = vk::DescriptorPoolCreateInfo::default()
            .max_sets(1)
            .pool_sizes(std::slice::from_ref(&screen_pool_size));
        let screen_pool = unsafe { device.create_descriptor_pool(&screen_pool_info, None) }
            .expect("Failed to create screen descriptor pool");

        let screen_set_alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(screen_pool)
            .set_layouts(std::slice::from_ref(&screen_dsl));
        let screen_set = unsafe { device.allocate_descriptor_sets(&screen_set_alloc_info) }
            .expect("Failed to allocate screen descriptor set")[0];

        let buffer_info_for_screen_set = vk::DescriptorBufferInfo::default()
            .buffer(screen_ubo_buffer.buffer)
            .offset(0)
            .range(std::mem::size_of::<ScreenInfo>() as vk::DeviceSize);
        let write_screen_set = vk::WriteDescriptorSet::default()
            .dst_set(screen_set)
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(std::slice::from_ref(&buffer_info_for_screen_set));
        unsafe {
            device.update_descriptor_sets(std::slice::from_ref(&write_screen_set), &[]);
        }

        // --- Quad Geometry Buffers ---
        let quad_vb = create_buffer_with_size(
            &device,
            &phys_mem_props,
            (std::mem::size_of::<Vertex>() * QUAD_VERTICES.len()) as vk::DeviceSize,
            Some(&QUAD_VERTICES),
            vk::BufferUsageFlags::VERTEX_BUFFER,
            MemoryLocation::GpuOnly, // Or CpuToGpu if static and small
        );
        let quad_ib = create_buffer_with_size(
            &device,
            &phys_mem_props,
            (std::mem::size_of::<u16>() * QUAD_INDICES.len()) as vk::DeviceSize,
            Some(&QUAD_INDICES),
            vk::BufferUsageFlags::INDEX_BUFFER,
            MemoryLocation::GpuOnly,
        );

        // --- Graphics Pipeline ---
        let vs_module = create_shader_module(&device, SPRITE_VERT_SHADER_BYTES)
            .expect("Failed to create vertex shader module");
        let fs_module = create_shader_module(&device, SPRITE_FRAG_SHADER_BYTES)
            .expect("Failed to create fragment shader module");
        let main_function_name = unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") };

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vs_module)
                .name(main_function_name),
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(fs_module)
                .name(main_function_name),
        ];

        let vertex_binding_descs = [
            vk::VertexInputBindingDescription {
                binding: 0,
                stride: std::mem::size_of::<Vertex>() as u32,
                input_rate: vk::VertexInputRate::VERTEX,
            },
            vk::VertexInputBindingDescription {
                binding: 1,
                stride: std::mem::size_of::<InstanceData>() as u32,
                input_rate: vk::VertexInputRate::INSTANCE,
            },
        ];
        let vertex_attr_descs = [
            // Vertex attributes (pos, uv) from binding 0
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: vk::Format::R32G32_SFLOAT,
                offset: 0,
            },
            vk::VertexInputAttributeDescription {
                location: 1,
                binding: 0,
                format: vk::Format::R32G32_SFLOAT,
                offset: 8,
            },
            // Instance attributes from binding 1
            vk::VertexInputAttributeDescription {
                location: 2,
                binding: 1,
                format: vk::Format::R32G32_SFLOAT,
                offset: 0,
            },
            vk::VertexInputAttributeDescription {
                location: 3,
                binding: 1,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: 8,
            }, // mat_col0
            vk::VertexInputAttributeDescription {
                location: 4,
                binding: 1,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: 20,
            }, // mat_col1
            vk::VertexInputAttributeDescription {
                location: 5,
                binding: 1,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: 32,
            }, // mat_col2
            vk::VertexInputAttributeDescription {
                location: 6,
                binding: 1,
                format: vk::Format::R32_SFLOAT,
                offset: 44,
            },
        ];
        let vi_state = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&vertex_binding_descs)
            .vertex_attribute_descriptions(&vertex_attr_descs);
        let ia_state = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);
        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1); // Dynamic
        let raster_state = vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false) // Standard
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::NONE) // No culling for 2D sprites usually
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE); // Or CLOCKWISE depending on quad winding
        let ms_state = vk::PipelineMultisampleStateCreateInfo::default()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE) // Or SRC_ALPHA depending on desired alpha composition
            .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA) // Or ZERO
            .alpha_blend_op(vk::BlendOp::ADD);
        let blend_state = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .attachments(std::slice::from_ref(&color_blend_attachment));

        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dyn_state_info =
            vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

        // Pipeline Layout: Set 0 for Screen UBO, Set 1 for Sprite Texture
        let set_layouts = [screen_dsl, sprite_texture_set_layout];
        let pl_create_info = vk::PipelineLayoutCreateInfo::default().set_layouts(&set_layouts);
        let pipeline_layout = unsafe { device.create_pipeline_layout(&pl_create_info, None) }
            .expect("Failed to create pipeline layout");

        let gp_create_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages)
            .vertex_input_state(&vi_state)
            .input_assembly_state(&ia_state)
            .viewport_state(&viewport_state)
            .rasterization_state(&raster_state)
            .multisample_state(&ms_state)
            .color_blend_state(&blend_state)
            .dynamic_state(&dyn_state_info)
            .layout(pipeline_layout)
            .render_pass(render_pass) // Use the render pass from GpuResourceManager
            .subpass(0);

        let pipeline = unsafe {
            device.create_graphics_pipelines(vk::PipelineCache::null(), &[gp_create_info], None)
        }
        .expect("Failed to create graphics pipeline")[0];

        unsafe {
            device.destroy_shader_module(vs_module, None);
            device.destroy_shader_module(fs_module, None);
        }

        self.device = Some(device); // Store the device
        self.phys_mem_props = Some(phys_mem_props);
        self.pipeline_layout = Some(pipeline_layout);
        self.pipeline = Some(pipeline);
        self.quad_vertex_buffer = Some(quad_vb);
        self.quad_index_buffer = Some(quad_ib);
        self.screen_info_buffer = Some(screen_ubo_buffer);
        self.screen_descriptor_set_layout = Some(screen_dsl);
        self.screen_descriptor_pool = Some(screen_pool);
        self.screen_descriptor_set = Some(screen_set);
        // Instance buffer will be created on first render or with initial capacity
        self.instance_capacity = 64; // Initial capacity, can grow
        self.ensure_instance_buffer_capacity(self.instance_capacity);
    }

    fn ensure_instance_buffer_capacity(&mut self, required_capacity: usize) {
        if required_capacity == 0 {
            return;
        }
        let device = self
            .device
            .as_ref()
            .expect("Device not available for instance buffer");
        let phys_mem = self
            .phys_mem_props
            .as_ref()
            .expect("Phys mem props not available");

        if required_capacity > self.instance_capacity || self.instance_buffer.is_none() {
            if let Some(old_buffer) = self.instance_buffer.take() {
                old_buffer.destroy(device);
            }
            // Grow capacity, e.g., to next power of two or by a fixed factor
            let new_capacity = required_capacity.max(self.instance_capacity * 2).max(64); // Ensure some minimum
            let buffer_size =
                (std::mem::size_of::<InstanceData>() * new_capacity) as vk::DeviceSize;

            self.instance_buffer = Some(create_buffer_with_size::<InstanceData>(
                device,
                phys_mem,
                buffer_size,
                None, // No initial data
                vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST, // For copying from staging
                MemoryLocation::GpuOnly, // Optimal for vertex data accessed by GPU
            ));
            self.instance_capacity = new_capacity;
        }
    }

    fn on_gpu_resources_lost_internal(&mut self, keep_device_ref: bool) {
        if let Some(device_ref) = &self.device {
            unsafe {
                if let Some(buffer) = self.instance_buffer.take() {
                    buffer.destroy(device_ref);
                }
                if let Some(buffer) = self.quad_vertex_buffer.take() {
                    buffer.destroy(device_ref);
                }
                if let Some(buffer) = self.quad_index_buffer.take() {
                    buffer.destroy(device_ref);
                }
                if let Some(buffer) = self.screen_info_buffer.take() {
                    buffer.destroy(device_ref);
                }

                if let Some(pipeline) = self.pipeline.take() {
                    device_ref.destroy_pipeline(pipeline, None);
                }
                if let Some(layout) = self.pipeline_layout.take() {
                    device_ref.destroy_pipeline_layout(layout, None);
                }

                // Descriptor sets are freed with the pool
                if let Some(pool) = self.screen_descriptor_pool.take() {
                    device_ref.destroy_descriptor_pool(pool, None);
                }
                self.screen_descriptor_set = None; // References a set from the now-destroyed pool
                if let Some(dsl) = self.screen_descriptor_set_layout.take() {
                    device_ref.destroy_descriptor_set_layout(dsl, None);
                }
            }
        }
        if !keep_device_ref {
            self.device = None;
            self.phys_mem_props = None;
        }
    }

    pub fn on_gpu_resources_lost(&mut self) {
        self.on_gpu_resources_lost_internal(false);
    }

    pub fn on_window_resized(&mut self, window_size: PhysicalSize<u32>, scale_factor: f64) {
        self.current_window_size = window_size;
        self.current_scale_factor = scale_factor;

        if let (Some(device), Some(screen_buffer)) =
            (self.device.as_ref(), self.screen_info_buffer.as_ref())
        {
            if let Some(mapped_ptr) = screen_buffer.mapped_ptr {
                let screen_info_data = ScreenInfo {
                    resolution: [window_size.width as f32, window_size.height as f32],
                    scale_factor: scale_factor as f32,
                    _padding: 0.0,
                };
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        &screen_info_data as *const _ as *const std::ffi::c_void,
                        mapped_ptr,
                        std::mem::size_of::<ScreenInfo>(),
                    );
                }
                // If not HOST_COHERENT, a flush would be needed here.
            } else {
                // Buffer not persistently mapped, would need staging buffer to update if not CpuToGpu Coherent
                // This example assumes CpuToGpu mapping is persistent and coherent.
                eprintln!("Warning: Screen UBO not mapped for resize update.");
            }
        }
    }

    pub fn render(
        &mut self,
        cmd_buffer: vk::CommandBuffer,
        // framebuffer: vk::Framebuffer, // This is implicitly handled by render pass begin from GpuResourceManager
        // render_area: vk::Rect2D,    // This is implicitly handled by render pass begin from GpuResourceManager
        viewport: vk::Viewport, // Renderer should set its own viewport/scissor if needed inside render pass
        scissor: vk::Rect2D,
        sprite_commands: &[SpriteRenderCommand<SA>],
    ) {
        if sprite_commands.is_empty() {
            return;
        }

        let device = self
            .device
            .as_ref()
            .expect("Device not available for rendering");
        let phys_mem = self
            .phys_mem_props
            .as_ref()
            .expect("Phys mem props not available");

        let required_capacity = sprite_commands.len();

        if required_capacity > 0 {
            let device = self
                .device
                .as_ref()
                .expect("Device not available for instance buffer");
            let phys_mem = self
                .phys_mem_props
                .as_ref()
                .expect("Phys mem props not available");

            if required_capacity > self.instance_capacity || self.instance_buffer.is_none() {
                if let Some(old_buffer) = self.instance_buffer.take() {
                    old_buffer.destroy(device);
                }
                // Grow capacity, e.g., to next power of two or by a fixed factor
                let new_capacity = required_capacity.max(self.instance_capacity * 2).max(64); // Ensure some minimum
                let buffer_size =
                    (std::mem::size_of::<InstanceData>() * new_capacity) as vk::DeviceSize;

                self.instance_buffer = Some(create_buffer_with_size::<InstanceData>(
                    device,
                    phys_mem,
                    buffer_size,
                    None, // No initial data
                    vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST, // For copying from staging
                    MemoryLocation::GpuOnly, // Optimal for vertex data accessed by GPU
                ));
                self.instance_capacity = new_capacity;
            }
        }

        let instance_gpu_buffer = self.instance_buffer.as_ref().unwrap();

        // --- Upload Instance Data via Staging Buffer ---
        let instance_data_cpu: Vec<InstanceData> = sprite_commands
            .iter()
            .map(|cmd| {
                let m = cmd.matrix.to_cols_array_2d(); // [[f32; 3]; 3]
                InstanceData {
                    size: [cmd.size.x, cmd.size.y],
                    matrix_col0: m[0],
                    matrix_col1: m[1],
                    matrix_col2: m[2],
                    alpha: cmd.alpha,
                    _padding_instance: [0.0, 0.0],
                }
            })
            .collect();

        let staging_buffer_size =
            (std::mem::size_of::<InstanceData>() * instance_data_cpu.len()) as vk::DeviceSize;
        let staging_buffer = create_buffer_with_size(
            device,
            phys_mem,
            staging_buffer_size,
            Some(&instance_data_cpu),
            vk::BufferUsageFlags::TRANSFER_SRC,
            MemoryLocation::CpuToGpu, // Create, map, copy, unmap (if not persistent)
        );

        let copy_region = vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size: staging_buffer.size,
        };
        unsafe {
            device.cmd_copy_buffer(
                cmd_buffer,
                staging_buffer.buffer,
                instance_gpu_buffer.buffer,
                &[copy_region],
            );
        }

        // Barrier for transfer completion before vertex input
        let buffer_barrier = vk::BufferMemoryBarrier::default()
            .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .dst_access_mask(vk::AccessFlags::VERTEX_ATTRIBUTE_READ)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED) // No ownership transfer
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .buffer(instance_gpu_buffer.buffer)
            .offset(0)
            .size(vk::WHOLE_SIZE); // Or staging_buffer.size if more precise
        unsafe {
            device.cmd_pipeline_barrier(
                cmd_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::VERTEX_INPUT,
                vk::DependencyFlags::empty(),
                &[],
                &[buffer_barrier],
                &[],
            );
        }

        // --- Actual Drawing ---
        // RenderPassBegin is handled by AshGpuResourceManager typically before calling engine.update,
        // or engine.update calls this render function within its own render pass.
        // This function assumes it's called *within* an active render pass.

        unsafe {
            device.cmd_bind_pipeline(
                cmd_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.unwrap(),
            );
            device.cmd_set_viewport(cmd_buffer, 0, &[viewport]);
            device.cmd_set_scissor(cmd_buffer, 0, &[scissor]);

            // Bind common resources
            device.cmd_bind_descriptor_sets(
                cmd_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout.unwrap(),
                0,
                &[self.screen_descriptor_set.unwrap()],
                &[], // Set 0: Screen UBO
            );
            device.cmd_bind_vertex_buffers(
                cmd_buffer,
                0,
                &[self.quad_vertex_buffer.as_ref().unwrap().buffer],
                &[0],
            ); // Quad vertices
            device.cmd_bind_vertex_buffers(cmd_buffer, 1, &[instance_gpu_buffer.buffer], &[0]); // Instance data
            device.cmd_bind_index_buffer(
                cmd_buffer,
                self.quad_index_buffer.as_ref().unwrap().buffer,
                0,
                vk::IndexType::UINT16,
            ); // Quad indices

            // Batch draw calls by descriptor set (sprite texture)
            let mut current_texture_set = vk::DescriptorSet::null();
            let mut batch_start_instance_index = 0u32;
            for (i, sprite_cmd) in sprite_commands.iter().enumerate() {
                if sprite_cmd.descriptor_set != current_texture_set {
                    // Draw previous batch if any
                    if i > batch_start_instance_index as usize {
                        device.cmd_draw_indexed(
                            cmd_buffer,
                            QUAD_INDICES.len() as u32, // indexCount
                            (i - batch_start_instance_index as usize) as u32, // instanceCount
                            0,                         // firstIndex
                            0,                         // vertexOffset
                            batch_start_instance_index, // firstInstance
                        );
                    }
                    // Start new batch
                    current_texture_set = sprite_cmd.descriptor_set;
                    batch_start_instance_index = i as u32;
                    device.cmd_bind_descriptor_sets(
                        cmd_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        self.pipeline_layout.unwrap(),
                        1,
                        &[current_texture_set],
                        &[], // Set 1: Sprite Texture
                    );
                }
            }
            // Draw the last batch
            if !sprite_commands.is_empty()
                && (sprite_commands.len() > batch_start_instance_index as usize)
            {
                device.cmd_draw_indexed(
                    cmd_buffer,
                    QUAD_INDICES.len() as u32,
                    (sprite_commands.len() - batch_start_instance_index as usize) as u32,
                    0,
                    0,
                    batch_start_instance_index,
                );
            }
        }
        // RenderPassEnd is handled by AshGpuResourceManager or engine.update

        staging_buffer.destroy(device); // Staging buffer can be destroyed after submit, but here is simpler for one-shot
    }
}

impl<SA: Copy + PartialEq> Drop for AshSpriteRenderer<SA> {
    fn drop(&mut self) {
        self.on_gpu_resources_lost_internal(true);
    }
}
