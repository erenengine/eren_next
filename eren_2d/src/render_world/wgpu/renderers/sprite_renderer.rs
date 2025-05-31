use std::collections::HashMap;

use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;

use crate::render_world::wgpu::asset_managers::sprite_asset_manager::WgpuTexture;

const SPRITE_SHADER: &str = include_str!("sprite.wgsl");

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ScreenInfo {
    resolution: [f32; 2],
}

pub struct SpriteRenderCommand {
    pub x: f32,
    pub y: f32,
    pub texture: WgpuTexture,
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

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct InstanceData {
    offset: [f32; 2],
    size: [f32; 2],
}

impl InstanceData {
    const ATTRIBS: [wgpu::VertexAttribute; 2] = wgpu::vertex_attr_array![
        2 => Float32x2, // offset
        3 => Float32x2  // size
    ];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceData>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
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

    screen_resolution_buffer: Option<wgpu::Buffer>,
    screen_bind_group: Option<wgpu::BindGroup>,

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

            screen_resolution_buffer: None,
            screen_bind_group: None,

            bind_group_cache: HashMap::new(),
        }
    }

    pub fn on_gpu_resources_ready(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        window_size: PhysicalSize<u32>,
    ) {
        // Take ownership clones so we can keep them around.
        self.device = Some(device.clone());
        self.queue = Some(queue.clone());

        // Screen resolution buffer ---------------------------------------------
        let screen_info = ScreenInfo {
            resolution: [window_size.width as f32, window_size.height as f32],
        };

        let screen_resolution_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("screen info buffer"),
                contents: bytemuck::cast_slice(&[screen_info]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let screen_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("screen bgl"),
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

        let screen_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("screen bind group"),
            layout: &screen_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: screen_resolution_buffer.as_entire_binding(),
            }],
        });

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
        let sprite_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            bind_group_layouts: &[&screen_bind_group_layout, &sprite_bind_group_layout],
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
        self.bind_group_layout = Some(sprite_bind_group_layout);
        self.pipeline = Some(pipeline);
        self.quad_vertex_buffer = Some(quad_vb);
        self.quad_index_buffer = Some(quad_ib);
        self.screen_resolution_buffer = Some(screen_resolution_buffer);
        self.screen_bind_group = Some(screen_bind_group);
        self.bind_group_cache.clear();
    }

    pub fn on_gpu_resources_lost(&mut self) {
        self.device = None;
        self.queue = None;
        self.pipeline = None;
        self.sampler = None;
        self.bind_group_layout = None;
        self.quad_vertex_buffer = None;
        self.quad_index_buffer = None;
        self.screen_resolution_buffer = None;
        self.screen_bind_group = None;
        self.bind_group_cache.clear();
    }

    pub fn on_window_resized(&mut self, window_size: PhysicalSize<u32>) {
        if let (Some(buffer), Some(queue)) =
            (self.screen_resolution_buffer.as_ref(), self.queue.as_ref())
        {
            let new_data = ScreenInfo {
                resolution: [window_size.width as f32, window_size.height as f32],
            };
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&[new_data]));
        }
    }

    pub fn render(
        &mut self,
        surface_texture_view: &wgpu::TextureView,
        command_encoder: &mut wgpu::CommandEncoder,
        render_commands: Vec<SpriteRenderCommand>,
    ) {
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

        let instance_data: Vec<InstanceData> = if render_commands.is_empty() {
            vec![InstanceData {
                offset: [0.0, 0.0],
                size: [1.0, 1.0],
            }] // dummy instance
        } else {
            render_commands
                .iter()
                .map(|cmd| InstanceData {
                    offset: [cmd.x, cmd.y],
                    size: [cmd.texture.width as f32, cmd.texture.height as f32],
                })
                .collect()
        };

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("instance buffer"),
            contents: bytemuck::cast_slice(&instance_data),
            usage: wgpu::BufferUsages::VERTEX,
        });

        rpass.set_pipeline(pipeline);
        rpass.set_vertex_buffer(0, vb.slice(..));
        rpass.set_vertex_buffer(1, instance_buffer.slice(..));
        rpass.set_index_buffer(ib.slice(..), wgpu::IndexFormat::Uint16);

        // Draw each sprite -----------------------------------------------------
        for (i, cmd) in render_commands.iter().enumerate() {
            // -- bind group cache keyed by texture_view pointer ---------------
            let key = &cmd.texture.view as *const _ as usize;
            let sprite_bind_group = self.bind_group_cache.entry(key).or_insert_with(|| {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("sprite bg"),
                    layout: bgl,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&cmd.texture.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(sampler),
                        },
                    ],
                })
            });

            rpass.set_bind_group(0, &self.screen_bind_group, &[]);
            rpass.set_bind_group(1, &sprite_bind_group.clone(), &[]);
            rpass.draw_indexed(0..6, 0, i as u32..(i as u32 + 1));
        }
    }
}
