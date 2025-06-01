use glam::{Mat3, Vec2};
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;

const SPRITE_SHADER: &str = include_str!("sprite.wgsl");

const BASE_COLOR: wgpu::Color = wgpu::Color {
    r: 0.1,
    g: 0.2,
    b: 0.3,
    a: 1.0,
};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ScreenInfo {
    resolution: [f32; 2],
    scale_factor: f32,
    _padding: f32,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    pos: [f32; 2],
    uv: [f32; 2],
}

impl Vertex {
    const ATTRIBUTES: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x2, // pos
        1 => Float32x2  // uv
    ];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceData {
    size: [f32; 2],
    matrix: [[f32; 3]; 3],
    alpha: f32,
}

impl InstanceData {
    const ATTRIBUTES: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
        2 => Float32x2,    // size
        3 => Float32x3,    // matrix.col0 (matrix[0])
        4 => Float32x3,    // matrix.col1 (matrix[1])
        5 => Float32x3,    // matrix.col2 (matrix[2])
        6 => Float32,      // alpha
    ];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceData>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

pub struct SpriteRenderCommand<SA> {
    pub size: Vec2,
    pub matrix: Mat3,
    pub alpha: f32,
    pub sprite_asset_id: SA,
    pub bind_group: wgpu::BindGroup,
}

pub struct WgpuSpriteRenderer<SA> {
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,

    pipeline: Option<wgpu::RenderPipeline>,
    quad_vertex_buffer: Option<wgpu::Buffer>,
    quad_index_buffer: Option<wgpu::Buffer>,
    screen_info_buffer: Option<wgpu::Buffer>,
    screen_info_bind_group: Option<wgpu::BindGroup>,

    instance_buffer: Option<wgpu::Buffer>,
    instance_buffer_capacity: usize,

    phantom: std::marker::PhantomData<SA>,
}

impl<SA: PartialEq + Copy> WgpuSpriteRenderer<SA> {
    pub fn new() -> Self {
        Self {
            device: None,
            queue: None,

            pipeline: None,
            quad_vertex_buffer: None,
            quad_index_buffer: None,
            screen_info_buffer: None,
            screen_info_bind_group: None,

            instance_buffer: None,
            instance_buffer_capacity: 0,

            phantom: std::marker::PhantomData,
        }
    }

    pub fn on_gpu_resources_ready(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        sprite_bind_group_layout: &wgpu::BindGroupLayout,
        window_size: PhysicalSize<u32>,
        window_scale_factor: f64,
    ) {
        self.device = Some(device.clone());
        self.queue = Some(queue.clone());

        let screen_info = ScreenInfo {
            resolution: [window_size.width as f32, window_size.height as f32],
            scale_factor: window_scale_factor as f32,
            _padding: 0.0,
        };

        let screen_info_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("screen info buffer"),
            contents: bytemuck::cast_slice(&[screen_info]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let screen_info_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("screen info bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let screen_info_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("screen info bind group"),
            layout: &screen_info_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: screen_info_buffer.as_entire_binding(),
            }],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sprite shader"),
            source: wgpu::ShaderSource::Wgsl(SPRITE_SHADER.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sprite pipeline layout"),
            bind_group_layouts: &[&screen_info_bind_group_layout, &sprite_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sprite pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main".into(),
                buffers: &[Vertex::desc(), InstanceData::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main".into(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let vertices: &[Vertex] = &[
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

        let indices: &[u16] = &[0, 1, 2, 2, 3, 0];

        let quad_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sprite quad vertex buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let quad_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sprite quad index buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        self.pipeline = Some(pipeline);
        self.quad_vertex_buffer = Some(quad_vertex_buffer);
        self.quad_index_buffer = Some(quad_index_buffer);
        self.screen_info_buffer = Some(screen_info_buffer);
        self.screen_info_bind_group = Some(screen_info_bind_group);
    }

    pub fn on_gpu_resources_lost(&mut self) {
        self.device = None;
        self.queue = None;

        self.pipeline = None;
        self.quad_vertex_buffer = None;
        self.quad_index_buffer = None;
        self.screen_info_buffer = None;
        self.screen_info_bind_group = None;

        self.instance_buffer = None;
        self.instance_buffer_capacity = 0;
    }

    pub fn on_window_resized(&mut self, window_size: PhysicalSize<u32>, window_scale_factor: f64) {
        if let (Some(buffer), Some(queue)) = (&self.screen_info_buffer, &self.queue) {
            let new_screen_info = ScreenInfo {
                resolution: [window_size.width as f32, window_size.height as f32],
                scale_factor: window_scale_factor as f32,
                _padding: 0.0,
            };
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&[new_screen_info]));
        }
    }

    pub fn render(
        &mut self,
        surface_texture_view: &wgpu::TextureView,
        command_encoder: &mut wgpu::CommandEncoder,
        render_commands: Vec<SpriteRenderCommand<SA>>,
    ) {
        let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("sprite render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: surface_texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(BASE_COLOR),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        if render_commands.is_empty() {
            return;
        }

        if let (
            Some(device),
            Some(queue),
            Some(pipeline),
            Some(quad_vertex_buffer),
            Some(quad_index_buffer),
            Some(screen_info_bind_group),
        ) = (
            &self.device,
            &self.queue,
            &self.pipeline,
            &self.quad_vertex_buffer,
            &self.quad_index_buffer,
            &self.screen_info_bind_group,
        ) {
            let instance_data_vec: Vec<InstanceData> = render_commands
                .iter()
                .map(|cmd| InstanceData {
                    size: [cmd.size.x, cmd.size.y],
                    matrix: cmd.matrix.to_cols_array_2d(),
                    alpha: cmd.alpha,
                })
                .collect();

            let num_instances = instance_data_vec.len();
            if num_instances > self.instance_buffer_capacity || self.instance_buffer.is_none() {
                if let Some(old_buffer) = self.instance_buffer.take() {
                    old_buffer.destroy();
                }

                self.instance_buffer_capacity = num_instances.next_power_of_two().max(16);

                let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("sprite instance buffer"),
                    size: (self.instance_buffer_capacity * std::mem::size_of::<InstanceData>())
                        as wgpu::BufferAddress,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                });

                queue.write_buffer(&new_buffer, 0, bytemuck::cast_slice(&instance_data_vec));

                self.instance_buffer = Some(new_buffer);
            } else {
                queue.write_buffer(
                    self.instance_buffer.as_ref().unwrap(),
                    0,
                    bytemuck::cast_slice(&instance_data_vec),
                );
            }

            let current_instance_buffer = self.instance_buffer.as_ref().unwrap();

            render_pass.set_pipeline(pipeline);
            render_pass.set_vertex_buffer(0, quad_vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(
                1,
                current_instance_buffer.slice(
                    ..(num_instances * std::mem::size_of::<InstanceData>()) as wgpu::BufferAddress,
                ),
            );
            render_pass.set_index_buffer(quad_index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.set_bind_group(0, screen_info_bind_group, &[]);

            let mut current_asset_id: Option<SA> = None;
            let mut batch_start = 0u32;

            for (i, cmd) in render_commands.iter().enumerate() {
                if current_asset_id != Some(cmd.sprite_asset_id) {
                    if i as u32 > batch_start {
                        render_pass.draw_indexed(0..6, 0, batch_start..i as u32);
                    }
                    render_pass.set_bind_group(1, &cmd.bind_group, &[]);
                    current_asset_id = Some(cmd.sprite_asset_id);
                    batch_start = i as u32;
                }
            }

            if render_commands.len() as u32 > batch_start {
                render_pass.draw_indexed(0..6, 0, batch_start..render_commands.len() as u32);
            }
        }
    }
}
