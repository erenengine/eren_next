use ash::{Device, vk};
use bytemuck::{Pod, Zeroable, cast_slice};
use eren_core::render_world::ash::buffer::{
    BufferResource, MemoryLocation, create_buffer_with_size,
};
use glam::{Mat3, Vec2};
use std::{ffi::CStr, marker::PhantomData};
use winit::dpi::PhysicalSize;

/// SPIR-V 셰이더 바이너리 (빌드 시 include_bytes! 매크로로 포함)
const SPRITE_VERT_SHADER_BYTES: &[u8] = include_bytes!("sprite.vert.spv");
const SPRITE_FRAG_SHADER_BYTES: &[u8] = include_bytes!("sprite.frag.spv");

/// 셰이더 모듈 생성 헬퍼
pub fn create_shader_module(device: &Device, code: &[u8]) -> Result<vk::ShaderModule, vk::Result> {
    assert_eq!(
        code.len() % 4,
        0,
        "SPIR-V bytecode must be aligned to 4 bytes"
    );
    let mut owned = Vec::with_capacity(code.len());
    owned.extend_from_slice(code);
    let code_u32 = cast_slice(&owned);
    let create_info = vk::ShaderModuleCreateInfo::default().code(code_u32);
    unsafe { device.create_shader_module(&create_info, None) }
}

/// ──── UBO, 정점, 인스턴스 구조체 정의 ────

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct ScreenInfo {
    pub resolution: [f32; 2],
    pub scale_factor: f32,
    _padding: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
}

const QUAD_VERTICES: [Vertex; 4] = [
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
const QUAD_INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct InstanceData {
    pub size: [f32; 2],
    pub matrix_col0: [f32; 3],
    pub matrix_col1: [f32; 3],
    pub matrix_col2: [f32; 3],
    pub alpha: f32,
    _padding_instance: [f32; 2],
}

/// 게임 쪽에서 전달할 렌더 커맨드
pub struct SpriteRenderCommand<SA> {
    pub size: Vec2,
    pub matrix: Mat3,
    pub alpha: f32,
    pub sprite_asset_id: SA,
    pub descriptor_set: vk::DescriptorSet,
}

/// ──── AshSpriteRenderer 정의 ────
///
/// - render_pass 관련 로직은 모두 엔진 쪽(Engine)에서 담당.
/// - 여기서는 인스턴스 버퍼 업데이트와 실제 Draw() 호출만 수행.
pub struct AshSpriteRenderer<SA>
where
    SA: Copy + PartialEq,
{
    device: Option<Device>,
    phys_mem_props: Option<vk::PhysicalDeviceMemoryProperties>,

    // 파이프라인, 레이아웃
    pipeline_layout: Option<vk::PipelineLayout>,
    pipeline: Option<vk::Pipeline>,

    // Quad 지오메트리
    quad_vertex_buffer: Option<BufferResource>,
    quad_index_buffer: Option<BufferResource>,

    // 인스턴스용 GPU 버퍼
    instance_buffer: Option<BufferResource>,
    instance_capacity: usize,

    // Screen UBO
    screen_info_buffer: Option<BufferResource>,
    screen_descriptor_set_layout: Option<vk::DescriptorSetLayout>,
    screen_descriptor_pool: Option<vk::DescriptorPool>,
    screen_descriptor_set: Option<vk::DescriptorSet>,

    // *업로드 전용* 커맨드 풀 (Host → GPU 복사용)
    upload_command_pool: Option<vk::CommandPool>,

    // 현재 윈도우 크기 등
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
            upload_command_pool: None,
            current_window_size: PhysicalSize {
                width: 0,
                height: 0,
            },
            current_scale_factor: 1.0,
            phantom: PhantomData,
        }
    }

    /// ▶ 엔진이 GPU 자원을 모두 준비한 뒤 호출
    pub fn on_gpu_resources_ready(
        &mut self,
        _instance_ash: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: Device,
        phys_mem_props: vk::PhysicalDeviceMemoryProperties,
        render_pass: vk::RenderPass,
        sprite_texture_set_layout: vk::DescriptorSetLayout,
        window_size: PhysicalSize<u32>,
        scale_factor: f64,
        initial_max_sprites: usize,
    ) {
        // 기존에 초기화된 적이 있으면 정리
        if self.device.is_some() {
            self.on_gpu_resources_lost_internal(false);
        }

        self.current_window_size = window_size;
        self.current_scale_factor = scale_factor;

        // ============ (1) Upload 전용 Command Pool 생성 ============
        // 예: RESET_COMMAND_BUFFER, TRANSIENT 비트만 사용
        let pool_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::TRANSIENT)
            .queue_family_index(
                /* graphics queue family index; 엔진 쪽에서 알려줘야 함 */ 0,
            );
        let upload_pool = unsafe { device.create_command_pool(&pool_info, None) }
            .expect("Failed to create upload command pool");
        self.upload_command_pool = Some(upload_pool);

        // ============ (2) Screen UBO 생성 & Descriptor Set (Set=0) ============
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
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            MemoryLocation::CpuToGpu,
        );
        let screen_dsl_binding = vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
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

        // ============ (3) Quad 지오메트리 버퍼 생성 ============
        let quad_vb = create_buffer_with_size(
            &device,
            &phys_mem_props,
            (std::mem::size_of::<Vertex>() * QUAD_VERTICES.len()) as vk::DeviceSize,
            Some(&QUAD_VERTICES),
            vk::BufferUsageFlags::VERTEX_BUFFER,
            MemoryLocation::CpuToGpu,
        );
        let quad_ib = create_buffer_with_size(
            &device,
            &phys_mem_props,
            (std::mem::size_of::<u16>() * QUAD_INDICES.len()) as vk::DeviceSize,
            Some(&QUAD_INDICES),
            vk::BufferUsageFlags::INDEX_BUFFER,
            MemoryLocation::CpuToGpu,
        );

        // ============ (4) Graphics Pipeline 생성 (render_pass 사용) ============
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
            },
            vk::VertexInputAttributeDescription {
                location: 4,
                binding: 1,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: 20,
            },
            vk::VertexInputAttributeDescription {
                location: 5,
                binding: 1,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: 32,
            },
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
            .scissor_count(1);
        let raster_state = vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::NONE)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE);
        let ms_state = vk::PipelineMultisampleStateCreateInfo::default()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .alpha_blend_op(vk::BlendOp::ADD);
        let blend_state = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .attachments(std::slice::from_ref(&color_blend_attachment));

        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dyn_state_info =
            vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

        // Pipeline Layout: Set0=Screen UBO, Set1=Sprite Texture
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
            .render_pass(render_pass) // 엔진이 넘겨준 render_pass 사용
            .subpass(0);

        let pipeline = unsafe {
            device.create_graphics_pipelines(vk::PipelineCache::null(), &[gp_create_info], None)
        }
        .expect("Failed to create graphics pipeline")[0];

        unsafe {
            device.destroy_shader_module(vs_module, None);
            device.destroy_shader_module(fs_module, None);
        }

        // ── 모든 리소스 구조체 필드에 저장 ──
        self.device = Some(device);
        self.phys_mem_props = Some(phys_mem_props);
        self.pipeline_layout = Some(pipeline_layout);
        self.pipeline = Some(pipeline);
        self.quad_vertex_buffer = Some(quad_vb);
        self.quad_index_buffer = Some(quad_ib);
        self.screen_info_buffer = Some(screen_ubo_buffer);
        self.screen_descriptor_set_layout = Some(screen_dsl);
        self.screen_descriptor_pool = Some(screen_pool);
        self.screen_descriptor_set = Some(screen_set);
        self.instance_capacity = 0;

        // 인스턴스 버퍼를 초기 용량만큼 맞추기
        self.ensure_instance_buffer_capacity(initial_max_sprites);
    }

    fn ensure_instance_buffer_capacity(&mut self, required_capacity: usize) {
        if required_capacity == 0 {
            return;
        }
        let device = self.device.as_ref().expect("Device not available");
        let phys_mem = self
            .phys_mem_props
            .as_ref()
            .expect("Physical device memory props not available");

        if required_capacity > self.instance_capacity || self.instance_buffer.is_none() {
            if let Some(old_buffer) = self.instance_buffer.take() {
                old_buffer.destroy(device);
            }
            let new_capacity = required_capacity.max(self.instance_capacity * 2).max(64);
            let buffer_size =
                (std::mem::size_of::<InstanceData>() * new_capacity) as vk::DeviceSize;

            self.instance_buffer = Some(create_buffer_with_size::<InstanceData>(
                device,
                phys_mem,
                buffer_size,
                None,
                vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
                MemoryLocation::GpuOnly,
            ));
            self.instance_capacity = new_capacity;
        }
    }

    fn on_gpu_resources_lost_internal(&mut self, keep_device_ref: bool) {
        if let Some(device_ref) = &self.device {
            unsafe {
                // (1) 인스턴스 + 지오메트리 + UBO + 파이프라인 정리
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

                if let Some(pool) = self.screen_descriptor_pool.take() {
                    device_ref.destroy_descriptor_pool(pool, None);
                }
                self.screen_descriptor_set = None;
                if let Some(dsl) = self.screen_descriptor_set_layout.take() {
                    device_ref.destroy_descriptor_set_layout(dsl, None);
                }

                // (2) 업로드용 전용 커맨드 풀 정리
                if let Some(upload_pool) = self.upload_command_pool.take() {
                    device_ref.destroy_command_pool(upload_pool, None);
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

        if let (Some(_device), Some(screen_buffer)) =
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
            } else {
                eprintln!("Warning: Screen UBO not mapped for resize update.");
            }
        }
    }

    /// ▶ “별도 전용” 커맨드 풀을 통해 CPU→GPU 업로드만 수행
    ///    메인 렌더링 커맨드 버퍼(엔진에서 기록)를 대신 쓰게 되면
    ///    staging_buffer.destroy 시점 오류가 생깁니다.
    pub fn update_instance_buffer(&mut self, sprite_commands: &[SpriteRenderCommand<SA>]) {
        if sprite_commands.is_empty() {
            return;
        }
        let device = self.device.as_ref().unwrap();
        let phys_mem = self.phys_mem_props.as_ref().unwrap();
        let upload_pool = self.upload_command_pool.unwrap();

        // (1) 필요한 인스턴스 버퍼 용량 확인 + 재생성
        let required_count = sprite_commands.len();
        if required_count > 0 {
            if required_count > self.instance_capacity || self.instance_buffer.is_none() {
                if let Some(old) = self.instance_buffer.take() {
                    old.destroy(device);
                }
                let new_capacity = required_count.max(self.instance_capacity * 2).max(64);
                let buffer_size =
                    (std::mem::size_of::<InstanceData>() * new_capacity) as vk::DeviceSize;
                self.instance_buffer = Some(create_buffer_with_size::<InstanceData>(
                    device,
                    phys_mem,
                    buffer_size,
                    None,
                    vk::BufferUsageFlags::VERTEX_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
                    MemoryLocation::GpuOnly,
                ));
                self.instance_capacity = new_capacity;
            }
        }
        let instance_gpu_buffer = self.instance_buffer.as_ref().unwrap();

        // (2) CPU에서 InstanceData Vec 생성
        let instance_data_cpu: Vec<InstanceData> = sprite_commands
            .iter()
            .map(|cmd| {
                let m = cmd.matrix.to_cols_array_2d();
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

        // (3) 스테이징 버퍼 생성 (HostVisible) & CPU 메모리에 데이터를 바로 복사
        let staging_size =
            (std::mem::size_of::<InstanceData>() * instance_data_cpu.len()) as vk::DeviceSize;
        let staging_buffer = create_buffer_with_size(
            device,
            phys_mem,
            staging_size,
            Some(&instance_data_cpu),
            vk::BufferUsageFlags::TRANSFER_SRC,
            MemoryLocation::CpuToGpu,
        );

        // (4) “별도 전용” 업로드 커맨드 버퍼를 할당 & 시작
        let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(upload_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        let upload_cmd = unsafe { device.allocate_command_buffers(&alloc_info) }
            .expect("Failed to allocate upload command buffer")[0];

        let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe {
            device
                .begin_command_buffer(upload_cmd, &begin_info)
                .unwrap();
        }

        // (5) 버퍼 복사 + 배리어를 “upload_cmd” 커맨드 버퍼에 기록
        let copy_region = vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size: staging_size,
        };
        unsafe {
            device.cmd_copy_buffer(
                upload_cmd,
                staging_buffer.buffer,
                instance_gpu_buffer.buffer,
                &[copy_region],
            );
        }
        let buffer_barrier = vk::BufferMemoryBarrier::default()
            .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .dst_access_mask(vk::AccessFlags::VERTEX_ATTRIBUTE_READ)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .buffer(instance_gpu_buffer.buffer)
            .offset(0)
            .size(vk::WHOLE_SIZE);
        unsafe {
            device.cmd_pipeline_barrier(
                upload_cmd,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::VERTEX_INPUT,
                vk::DependencyFlags::empty(),
                &[],
                &[buffer_barrier],
                &[],
            );
        }

        unsafe {
            device.end_command_buffer(upload_cmd).unwrap();
        }

        // (6) Submit + queue_wait_idle → staging 버퍼 안전하게 destroy
        let submit_info =
            vk::SubmitInfo::default().command_buffers(std::slice::from_ref(&upload_cmd));
        unsafe {
            // graphics queue (engine 쪽에서 넘겨준 그래픽스 큐) 사용
            let graphics_queue = device.get_device_queue(/* index */ 0, 0);
            device
                .queue_submit(graphics_queue, &[submit_info], vk::Fence::null())
                .expect("Failed to submit upload command buffer");
            device.queue_wait_idle(graphics_queue).unwrap();
        }
        staging_buffer.destroy(device);
        unsafe {
            device.free_command_buffers(upload_pool, &[upload_cmd]);
        }
        // ─────────────────────────────────────────────────────────────────
        // 이제 “instance_gpu_buffer”엔 인스턴스 데이터가 GPU에 올라간 상태
        // 메인 렌더 패스에선 그냥 이 버퍼를 Vertex Input으로 바인딩만 하면 됨!
        // ─────────────────────────────────────────────────────────────────
    }

    /// ▶ 메인 렌더 패스가 이미 `vkCmdBeginRenderPass` 된 상태에서 호출
    pub fn draw(
        &self,
        cmd_buffer: vk::CommandBuffer,
        _render_pass: vk::RenderPass,
        _framebuffer: vk::Framebuffer,
        viewport: vk::Viewport,
        scissor: vk::Rect2D,
        sprite_commands: &[SpriteRenderCommand<SA>],
    ) {
        if sprite_commands.is_empty() {
            return;
        }
        let device = self.device.as_ref().unwrap();

        unsafe {
            device.cmd_bind_pipeline(
                cmd_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.unwrap(),
            );
            device.cmd_set_viewport(cmd_buffer, 0, &[viewport]);
            device.cmd_set_scissor(cmd_buffer, 0, &[scissor]);

            // Set 0: Screen UBO
            device.cmd_bind_descriptor_sets(
                cmd_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline_layout.unwrap(),
                0,
                &[self.screen_descriptor_set.unwrap()],
                &[],
            );
            // Quad 버퍼
            device.cmd_bind_vertex_buffers(
                cmd_buffer,
                0,
                &[self.quad_vertex_buffer.as_ref().unwrap().buffer],
                &[0],
            );
            // 인스턴스 버퍼만 바인딩 (+ offset=0)
            device.cmd_bind_vertex_buffers(
                cmd_buffer,
                1,
                &[self.instance_buffer.as_ref().unwrap().buffer],
                &[0],
            );
            device.cmd_bind_index_buffer(
                cmd_buffer,
                self.quad_index_buffer.as_ref().unwrap().buffer,
                0,
                vk::IndexType::UINT16,
            );

            // 텍스처 세트가 바뀔 때마다 DrawIndexed 배치 호출
            let mut current_texture_set = vk::DescriptorSet::null();
            let mut batch_start = 0u32;
            for (i, sprite_cmd) in sprite_commands.iter().enumerate() {
                if sprite_cmd.descriptor_set != current_texture_set {
                    if i > batch_start as usize {
                        device.cmd_draw_indexed(
                            cmd_buffer,
                            QUAD_INDICES.len() as u32,
                            (i - batch_start as usize) as u32,
                            0,
                            0,
                            batch_start,
                        );
                    }
                    current_texture_set = sprite_cmd.descriptor_set;
                    batch_start = i as u32;
                    device.cmd_bind_descriptor_sets(
                        cmd_buffer,
                        vk::PipelineBindPoint::GRAPHICS,
                        self.pipeline_layout.unwrap(),
                        1,
                        &[current_texture_set],
                        &[],
                    );
                }
            }
            let total = sprite_commands.len() as u32;
            if total > batch_start {
                device.cmd_draw_indexed(
                    cmd_buffer,
                    QUAD_INDICES.len() as u32,
                    total - batch_start,
                    0,
                    0,
                    batch_start,
                );
            }
        }
    }
}

impl<SA: Copy + PartialEq> Drop for AshSpriteRenderer<SA> {
    fn drop(&mut self) {
        self.on_gpu_resources_lost_internal(true);
    }
}
