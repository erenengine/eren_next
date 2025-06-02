use std::marker::PhantomData;

use ash::vk;
use glam::{Mat3, Vec2};

const SPRITE_VERT_SHADER_BYTES: &[u8] = include_bytes!("sprite.vert.spv");
const SPRITE_FRAG_SHADER_BYTES: &[u8] = include_bytes!("sprite.frag.spv");

const BASE_CLEAR_COLOR: vk::ClearValue = vk::ClearValue {
    color: vk::ClearColorValue {
        float32: [0.1, 0.2, 0.3, 1.0],
    },
};

fn create_shader_module(device: &ash::Device, code: &[u8]) -> Result<vk::ShaderModule, vk::Result> {
    let shader_code_u32 = ash::util::read_spv(&mut std::io::Cursor::new(code))
        .expect("Failed to read SPIR-V shader code");

    let shader_module_info = vk::ShaderModuleCreateInfo::default().code(&shader_code_u32);
    unsafe { device.create_shader_module(&shader_module_info, None) }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ScreenInfo {
    pub resolution: [f32; 2],
    pub scale_factor: f32,
    _padding: f32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceData {
    pub size: [f32; 2],
    pub matrix: [[f32; 3]; 3],
    pub alpha: f32,
}

pub struct SpriteRenderCommand<SA> {
    pub size: Vec2,
    pub matrix: Mat3,
    pub alpha: f32,
    pub sprite_asset_id: SA,
    pub descriptor_set: vk::DescriptorSet,
}

struct BufferResource {
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    size: vk::DeviceSize,
}

impl BufferResource {
    fn destroy(&self, device: &ash::Device) {
        unsafe {
            device.destroy_buffer(self.buffer, None);
            device.free_memory(self.memory, None);
        }
    }
}

enum MemoryLocation {
    GpuOnly,
    CpuToGpu,
}

fn create_buffer<T: bytemuck::Pod>(
    device: &ash::Device,
    phys: &vk::PhysicalDeviceMemoryProperties,
    contents: Option<&[T]>,
    usage: vk::BufferUsageFlags,
    location: MemoryLocation,
) -> BufferResource {
    let byte_len =
        (contents.map(|c| c.len()).unwrap_or(1) * std::mem::size_of::<T>()) as vk::DeviceSize;

    let buffer_create_info = vk::BufferCreateInfo::default()
        .usage(usage)
        .size(byte_len)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    let buffer = unsafe { device.create_buffer(&buffer_create_info, None) }.unwrap();

    let requirements = unsafe { device.get_buffer_memory_requirements(buffer) };

    let mem_type_index = (0..phys.memory_type_count)
        .find(|&i| {
            (requirements.memory_type_bits & (1 << i)) != 0
                && phys.memory_types[i as usize]
                    .property_flags
                    .contains(match location {
                        MemoryLocation::GpuOnly => vk::MemoryPropertyFlags::DEVICE_LOCAL,
                        MemoryLocation::CpuToGpu => {
                            vk::MemoryPropertyFlags::HOST_VISIBLE
                                | vk::MemoryPropertyFlags::HOST_COHERENT
                        }
                    })
        })
        .expect("No suitable memory type found!");

    let alloc_info = vk::MemoryAllocateInfo::default()
        .allocation_size(requirements.size)
        .memory_type_index(mem_type_index as _);

    let memory = unsafe { device.allocate_memory(&alloc_info, None) }.unwrap();
    unsafe { device.bind_buffer_memory(buffer, memory, 0) }.unwrap();

    if let (Some(data), MemoryLocation::CpuToGpu) = (contents, location) {
        unsafe {
            let ptr = device
                .map_memory(memory, 0, byte_len, vk::MemoryMapFlags::empty())
                .unwrap();
            std::ptr::copy_nonoverlapping(
                data.as_ptr() as *const std::ffi::c_void,
                ptr,
                byte_len as usize,
            );
            device.unmap_memory(memory);
        }
    }

    BufferResource {
        buffer,
        memory,
        size: byte_len,
    }
}

pub struct AshSpriteRenderer<SA> {
    device: ash::Device,

    phys_mem_props: vk::PhysicalDeviceMemoryProperties,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    render_pass: vk::RenderPass,
    descriptor_pool: vk::DescriptorPool,
    screen_info_descriptor_set: vk::DescriptorSet,

    quad_vbuffer: BufferResource,
    quad_ibuffer: BufferResource,

    screen_info_buffer: BufferResource,
    instance_buffer: Option<BufferResource>,
    instance_capacity: usize,

    phantom: PhantomData<SA>,
}

impl<SA: Copy + PartialEq> AshSpriteRenderer<SA> {
    pub fn new(
        instance: &ash::Instance,
        phys_device: vk::PhysicalDevice,
        device: ash::Device,
        swapchain_format: vk::Format,
        sprite_set_layout: vk::DescriptorSetLayout,
        window_size: winit::dpi::PhysicalSize<u32>,
        scale_factor: f64,
    ) -> Self {
        let phys_mem_props = unsafe { instance.get_physical_device_memory_properties(phys_device) };

        let screen_info_cpu = ScreenInfo {
            resolution: [window_size.width as f32, window_size.height as f32],
            scale_factor: scale_factor as f32,
            _padding: 0.0,
        };

        let screen_info_buffer = create_buffer(
            &device,
            &phys_mem_props,
            Some(std::slice::from_ref(&screen_info_cpu)),
            vk::BufferUsageFlags::UNIFORM_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
            MemoryLocation::CpuToGpu,
        );

        let pool_sizes = [vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 1,
        }];

        let descriptor_pool_info = vk::DescriptorPoolCreateInfo::default()
            .max_sets(1)
            .pool_sizes(&pool_sizes);

        let descriptor_pool =
            unsafe { device.create_descriptor_pool(&descriptor_pool_info, None) }.unwrap();

        let screen_set_layout_bindings = [vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX)];

        let screen_set_layout_info =
            vk::DescriptorSetLayoutCreateInfo::default().bindings(&screen_set_layout_bindings);

        let screen_set_layout =
            unsafe { device.create_descriptor_set_layout(&screen_set_layout_info, None) }.unwrap();

        let layouts = [screen_set_layout];

        let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&layouts);

        let screen_info_descriptor_set =
            unsafe { device.allocate_descriptor_sets(&alloc_info) }.unwrap()[0];

        let buffer_info = vk::DescriptorBufferInfo::default()
            .buffer(screen_info_buffer.buffer)
            .range(screen_info_buffer.size);

        let write = vk::WriteDescriptorSet::default()
            .dst_set(screen_info_descriptor_set)
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(std::slice::from_ref(&buffer_info));

        unsafe { device.update_descriptor_sets(&[write], &[]) };

        let vertices: [Vertex; 4] = [
            Vertex {
                pos: [-0.5, -0.5],
                uv: [0.0, 1.0],
            },
            Vertex {
                pos: [0.5, -0.5],
                uv: [1.0, 1.0],
            },
            Vertex {
                pos: [0.5, 0.5],
                uv: [1.0, 0.0],
            },
            Vertex {
                pos: [-0.5, 0.5],
                uv: [0.0, 0.0],
            },
        ];

        let indices: [u16; 6] = [0, 1, 2, 2, 3, 0];

        let quad_vbuffer = create_buffer(
            &device,
            &phys_mem_props,
            Some(&vertices),
            vk::BufferUsageFlags::VERTEX_BUFFER,
            MemoryLocation::CpuToGpu,
        );

        let quad_ibuffer = create_buffer(
            &device,
            &phys_mem_props,
            Some(&indices),
            vk::BufferUsageFlags::INDEX_BUFFER,
            MemoryLocation::CpuToGpu,
        );

        let color_attachment = vk::AttachmentDescription::default()
            .format(swapchain_format)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .samples(vk::SampleCountFlags::TYPE_1);

        let color_ref = vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };

        let subpass = vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(std::slice::from_ref(&color_ref));

        let render_pass_info = vk::RenderPassCreateInfo::default()
            .attachments(std::slice::from_ref(&color_attachment))
            .subpasses(std::slice::from_ref(&subpass));

        let render_pass = unsafe { device.create_render_pass(&render_pass_info, None) }.unwrap();

        let layouts = [screen_set_layout, sprite_set_layout];

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default().set_layouts(&layouts);

        let pipeline_layout =
            unsafe { device.create_pipeline_layout(&pipeline_layout_info, None) }.unwrap();

        let vs_module = create_shader_module(&device, SPRITE_VERT_SHADER_BYTES).unwrap();
        let fs_module = create_shader_module(&device, SPRITE_FRAG_SHADER_BYTES).unwrap();

        let entry_point = std::ffi::CStr::from_bytes_with_nul(b"main\0").unwrap();
        let shader_stages = [
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vs_module)
                .name(entry_point),
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(fs_module)
                .name(entry_point),
        ];

        let vertex_binding_descriptions = [
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

        let vertex_attribute_descriptions = [
            // Vertex::pos
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                format: vk::Format::R32G32_SFLOAT,
                offset: 0,
            },
            // Vertex::uv
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: vk::Format::R32G32_SFLOAT,
                offset: 8,
            },
            // InstanceData::size
            vk::VertexInputAttributeDescription {
                binding: 1,
                location: 2,
                format: vk::Format::R32G32_SFLOAT,
                offset: 0,
            },
            // InstanceData::matrix (3x3 packed as 3 vec3)
            vk::VertexInputAttributeDescription {
                binding: 1,
                location: 3,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: 8,
            },
            vk::VertexInputAttributeDescription {
                binding: 1,
                location: 4,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: 20,
            },
            vk::VertexInputAttributeDescription {
                binding: 1,
                location: 5,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: 32,
            },
            // InstanceData::alpha
            vk::VertexInputAttributeDescription {
                binding: 1,
                location: 6,
                format: vk::Format::R32_SFLOAT,
                offset: 44,
            },
        ];

        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&vertex_binding_descriptions)
            .vertex_attribute_descriptions(&vertex_attribute_descriptions);

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);

        let raster = vk::PipelineRasterizationStateCreateInfo::default()
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(vk::CullModeFlags::NONE)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE);

        let multisample = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD)
            .color_write_mask(vk::ColorComponentFlags::RGBA);

        let color_blend_state = vk::PipelineColorBlendStateCreateInfo::default()
            .attachments(std::slice::from_ref(&color_blend_attachment));

        let dynamic_state = vk::PipelineDynamicStateCreateInfo::default()
            .dynamic_states(&[vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR]);

        // Finally build graphics pipeline ------------------------------------------------------
        let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_state)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&raster)
            .multisample_state(&multisample)
            .color_blend_state(&color_blend_state)
            .dynamic_state(&dynamic_state)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0);

        let pipeline = unsafe {
            device.create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
        }
        .expect("Failed to create graphics pipeline")[0];

        // Cleanup shader modules (no longer needed after pipeline creation)
        unsafe {
            device.destroy_shader_module(vs_module, None);
            device.destroy_shader_module(fs_module, None);
        }

        Self {
            device,
            phys_mem_props,
            pipeline_layout,
            pipeline,
            render_pass,
            descriptor_pool,
            screen_info_descriptor_set,
            quad_vbuffer,
            quad_ibuffer,
            screen_info_buffer,
            instance_buffer: None,
            instance_capacity: 0,
            phantom: PhantomData,
        }
    }

    pub fn on_window_resized(&self, new_size: winit::dpi::PhysicalSize<u32>, scale_factor: f64) {
        let new_info = ScreenInfo {
            resolution: [new_size.width as f32, new_size.height as f32],
            scale_factor: scale_factor as f32,
            _padding: 0.0,
        };
        unsafe {
            let ptr = self
                .device
                .map_memory(
                    self.screen_info_buffer.memory,
                    0,
                    self.screen_info_buffer.size,
                    vk::MemoryMapFlags::empty(),
                )
                .unwrap();
            std::ptr::copy_nonoverlapping(
                &new_info as *const _ as *const u8,
                ptr as *mut u8,
                std::mem::size_of::<ScreenInfo>(),
            );
            self.device.unmap_memory(self.screen_info_buffer.memory);
        }
    }

    pub fn render(
        &mut self,
        cb: vk::CommandBuffer,
        framebuffer: vk::Framebuffer,
        render_area: vk::Rect2D,
        viewport: vk::Viewport,
        scissor: vk::Rect2D,
        commands: &[SpriteRenderCommand<SA>],
    ) {
        if commands.is_empty() {
            return;
        }

        if commands.len() > self.instance_capacity {
            if let Some(old) = self.instance_buffer.take() {
                old.destroy(&self.device);
            }
            self.instance_capacity = commands.len().next_power_of_two().max(16);
            self.instance_buffer = Some(create_buffer::<InstanceData>(
                &self.device,
                &self.phys_mem_props,
                None,
                vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
                MemoryLocation::GpuOnly,
            ));
        }

        let instance_buf = self.instance_buffer.as_ref().unwrap();

        // Upload per‑sprite instance data via a staging buffer
        let instance_data: Vec<InstanceData> = commands
            .iter()
            .map(|cmd| InstanceData {
                size: [cmd.size.x, cmd.size.y],
                matrix: cmd.matrix.to_cols_array_2d(),
                alpha: cmd.alpha,
            })
            .collect();

        let staging = create_buffer(
            &self.device,
            &self.phys_mem_props,
            Some(&instance_data),
            vk::BufferUsageFlags::TRANSFER_SRC,
            MemoryLocation::CpuToGpu,
        );

        // Record copy cmd (caller provided CB is already begun)
        let copy_region = vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size: staging.size,
        };
        unsafe {
            self.device
                .cmd_copy_buffer(cb, staging.buffer, instance_buf.buffer, &[copy_region])
        };

        // We rely on pipeline barrier outside (usually automatic via synchronisation‑graph)

        let rp_begin = vk::RenderPassBeginInfo::default()
            .render_pass(self.render_pass)
            .framebuffer(framebuffer)
            .render_area(render_area)
            .clear_values(std::slice::from_ref(&BASE_CLEAR_COLOR));

        unsafe {
            self.device
                .cmd_begin_render_pass(cb, &rp_begin, vk::SubpassContents::INLINE)
        };

        unsafe {
            self.device
                .cmd_bind_pipeline(cb, vk::PipelineBindPoint::GRAPHICS, self.pipeline);
            self.device.cmd_bind_descriptor_sets(
                cb,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout,
                0,
                &[self.screen_info_descriptor_set],
                &[],
            );
            self.device
                .cmd_bind_vertex_buffers(cb, 0, &[self.quad_vbuffer.buffer], &[0]);
            self.device
                .cmd_bind_vertex_buffers(cb, 1, &[instance_buf.buffer], &[0]);
            self.device.cmd_bind_index_buffer(
                cb,
                self.quad_ibuffer.buffer,
                0,
                vk::IndexType::UINT16,
            );
            self.device.cmd_set_viewport(cb, 0, &[viewport]);
            self.device.cmd_set_scissor(cb, 0, &[scissor]);
        }

        let mut current_set: Option<vk::DescriptorSet> = None;
        let mut batch_start = 0u32;

        for (i, cmd) in commands.iter().enumerate() {
            if current_set != Some(cmd.descriptor_set) {
                // flush previous batch
                if i as u32 > batch_start {
                    unsafe {
                        self.device.cmd_draw_indexed(
                            cb,
                            6,
                            i as u32 - batch_start,
                            0,
                            0,
                            batch_start,
                        );
                    }
                }
                // bind new descriptor set (set = 1)
                unsafe {
                    self.device.cmd_bind_descriptor_sets(
                        cb,
                        vk::PipelineBindPoint::GRAPHICS,
                        self.pipeline_layout,
                        1,
                        &[cmd.descriptor_set],
                        &[],
                    );
                }
                current_set = Some(cmd.descriptor_set);
                batch_start = i as u32;
            }
        }

        // flush final batch
        if commands.len() as u32 > batch_start {
            unsafe {
                self.device.cmd_draw_indexed(
                    cb,
                    6,
                    commands.len() as u32 - batch_start,
                    0,
                    0,
                    batch_start,
                )
            };
        }

        unsafe { self.device.cmd_end_render_pass(cb) };

        // Staging buffer cleaned up after submit, caller responsibility (or you can integrate
        // an arena/allocator).  For brevity we just destroy now (requires host‑idle)
        staging.destroy(&self.device);
    }
}

impl<SA> Drop for AshSpriteRenderer<SA> {
    fn drop(&mut self) {
        unsafe {
            if let Some(inst_buf) = self.instance_buffer.take() {
                inst_buf.destroy(&self.device);
            }
            self.quad_vbuffer.destroy(&self.device);
            self.quad_ibuffer.destroy(&self.device);
            self.screen_info_buffer.destroy(&self.device);

            self.device.destroy_pipeline(self.pipeline, None);
            self.device
                .destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_render_pass(self.render_pass, None);
            self.device
                .destroy_descriptor_pool(self.descriptor_pool, None);
        }
    }
}
