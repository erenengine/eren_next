use std::collections::HashMap;

use wgpu::util::DeviceExt;

const SPRITE_SHADER: &str = include_str!("sprite.wgsl");

pub struct SpriteRenderCommand {
    pub x: f32,
    pub y: f32,
    pub texture_view: wgpu::TextureView,
}

/// CPU side representation of one vertex in our unit quad.
#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    pos: [f32; 2],
    uv: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        0 => Float32x2, // pos
        1 => Float32x2  // uv
    ];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub struct WgpuSpriteRenderer {
    // cached GPU objects
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    pipeline: Option<wgpu::RenderPipeline>,
    sampler: Option<wgpu::Sampler>,
    bind_group_layout: Option<wgpu::BindGroupLayout>,
    quad_vertex_buffer: Option<wgpu::Buffer>,
    quad_index_buffer: Option<wgpu::Buffer>,

    // bind group cache for texture views (keyed by raw pointer)
    bind_group_cache: HashMap<usize, wgpu::BindGroup>,
}

impl WgpuSpriteRenderer {
    pub fn new() -> Self {
        Self {
            device: None,
            queue: None,
            pipeline: None,
            sampler: None,
            bind_group_layout: None,
            quad_vertex_buffer: None,
            quad_index_buffer: None,

            bind_group_cache: HashMap::new(),
        }
    }

    pub fn on_gpu_resources_ready(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        // Take ownership clones so we can keep them around.
        self.device = Some(device.clone());
        self.queue = Some(queue.clone());

        // Sampler --------------------------------------------------------------
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("sprite sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Bind group layout ----------------------------------------------------
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sprite bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Shader module --------------------------------------------------------
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sprite shader"),
            source: wgpu::ShaderSource::Wgsl(SPRITE_SHADER.into()),
        });

        // Render pipeline ------------------------------------------------------
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sprite pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sprite pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main".into(),
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main".into(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8UnormSrgb, // **change to swap chain fmt**
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

        // Unit quad geometry ---------------------------------------------------
        let vertices: &[f32] = &[
            // positions   // uvs
            -0.5, -0.5, 0.0, 1.0, 0.5, -0.5, 1.0, 1.0, 0.5, 0.5, 1.0, 0.0, -0.5, 0.5, 0.0, 0.0,
        ];
        let indices: &[u16] = &[0, 1, 2, 2, 3, 0];

        let quad_vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sprite quad vb"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let quad_ib = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sprite quad ib"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Store ----------------------------------------------------------------
        self.sampler = Some(sampler);
        self.bind_group_layout = Some(bind_group_layout);
        self.pipeline = Some(pipeline);
        self.quad_vertex_buffer = Some(quad_vb);
        self.quad_index_buffer = Some(quad_ib);
        self.bind_group_cache.clear();
    }

    pub fn on_gpu_resources_lost(&mut self) {}

    pub fn render(
        &mut self,
        surface_texture_view: &wgpu::TextureView,
        command_encoder: &mut wgpu::CommandEncoder,
        render_commands: Vec<SpriteRenderCommand>,
    ) {
        if render_commands.is_empty() {
            return;
        }

        // ---------------------------------------------------------------------
        let device = self.device.as_ref().expect("renderer not ready");
        let sampler = self.sampler.as_ref().unwrap();
        let bgl = self.bind_group_layout.as_ref().unwrap();
        let vb = self.quad_vertex_buffer.as_ref().unwrap();
        let ib = self.quad_index_buffer.as_ref().unwrap();
        let pipeline = self.pipeline.as_ref().unwrap();

        // Begin render pass ----------------------------------------------------
        let mut rpass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("sprite pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: surface_texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        rpass.set_pipeline(pipeline);
        rpass.set_vertex_buffer(0, vb.slice(..));
        rpass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint16);

        // Draw each sprite -----------------------------------------------------
        for cmd in render_commands {
            // -- bind group cache keyed by texture_view pointer ---------------
            let key = &cmd.texture_view as *const _ as usize;
            let bind_group = self.bind_group_cache.entry(key).or_insert_with(|| {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("sprite bg"),
                    layout: bgl,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&cmd.texture_view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(sampler),
                        },
                    ],
                })
            });

            rpass.set_bind_group(0, &bind_group.clone(), &[]);

            // Draw the quad ----------------------------------------------------
            rpass.draw_indexed(0..6, 0, 0..1);
        }
    }
}
