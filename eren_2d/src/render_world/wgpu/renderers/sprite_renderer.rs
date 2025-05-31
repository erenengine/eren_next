// WgpuSpriteRenderer 파일 (renderers/sprite_renderer.rs)

use std::collections::HashMap;
use std::hash::Hash; // 추가

use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;

use crate::render_world::wgpu::asset_managers::sprite_asset_manager::WgpuTexture;

const SPRITE_SHADER: &str = include_str!("sprite.wgsl");

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct ScreenInfo {
    resolution: [f32; 2],
    scale_factor: f32,
    _padding: f32,
}

// SpriteRenderCommand는 이제 SA 제네릭을 가집니다.
pub struct SpriteRenderCommand<SA> {
    pub x: f32,
    pub y: f32,
    pub sprite_asset_id: SA,  // 이 ID를 키로 사용합니다.
    pub texture: WgpuTexture, // 여전히 텍스처 자체는 필요합니다 (bind group 생성 시 view 접근).
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
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

// TextureKey 구조체는 이제 사용하지 않으므로 제거합니다.

pub struct WgpuSpriteRenderer<SA>
// SA 제네릭 추가
where
    SA: Eq + Hash + Copy + Ord, // bind_group_cache의 키로 사용하기 위한 제약
{
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    pipeline: Option<wgpu::RenderPipeline>,
    sampler: Option<wgpu::Sampler>,
    sprite_bind_group_layout: Option<wgpu::BindGroupLayout>,
    quad_vertex_buffer: Option<wgpu::Buffer>,
    quad_index_buffer: Option<wgpu::Buffer>,

    screen_resolution_buffer: Option<wgpu::Buffer>,
    screen_bind_group: Option<wgpu::BindGroup>,

    instance_buffer: Option<wgpu::Buffer>,
    instance_buffer_capacity: usize,

    bind_group_cache: HashMap<SA, wgpu::BindGroup>, // 키 타입을 SA로 변경
}

impl<SA> WgpuSpriteRenderer<SA>
// SA 제네릭 추가
where
    SA: Eq + Hash + Copy + Ord, // 생성자 및 다른 메서드에서도 필요할 수 있음
{
    pub fn new() -> Self {
        Self {
            device: None,
            queue: None,
            pipeline: None,
            sampler: None,
            sprite_bind_group_layout: None,
            quad_vertex_buffer: None,
            quad_index_buffer: None,
            screen_resolution_buffer: None,
            screen_bind_group: None,
            instance_buffer: None,
            instance_buffer_capacity: 0,
            bind_group_cache: HashMap::new(),
        }
    }

    pub fn on_gpu_resources_ready(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
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

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("sprite sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

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

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sprite shader"),
            source: wgpu::ShaderSource::Wgsl(SPRITE_SHADER.into()),
        });

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

        let quad_vb = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sprite quad vb"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let quad_ib = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sprite quad ib"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        self.sampler = Some(sampler);
        self.sprite_bind_group_layout = Some(sprite_bind_group_layout);
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
        self.sprite_bind_group_layout = None;
        self.quad_vertex_buffer = None;
        self.quad_index_buffer = None;
        self.screen_resolution_buffer = None;
        self.screen_bind_group = None;
        self.instance_buffer = None;
        self.instance_buffer_capacity = 0;
        self.bind_group_cache.clear();
    }

    pub fn on_window_resized(&mut self, window_size: PhysicalSize<u32>, window_scale_factor: f64) {
        if let (Some(buffer), Some(queue)) =
            (self.screen_resolution_buffer.as_ref(), self.queue.as_ref())
        {
            let new_data = ScreenInfo {
                resolution: [window_size.width as f32, window_size.height as f32],
                scale_factor: window_scale_factor as f32,
                _padding: 0.0,
            };
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&[new_data]));
        }
    }

    // render 함수도 SA 제네릭을 받고, 트레이트 제약을 추가합니다.
    pub fn render(
        // 충돌을 피하기 위해 다른 제네릭 이름 사용 (또는 SA가 Copy라면 그냥 SA)
        &mut self,
        surface_texture_view: &wgpu::TextureView,
        command_encoder: &mut wgpu::CommandEncoder,
        mut render_commands: Vec<SpriteRenderCommand<SA>>,
    ) {
        // render 함수 시그니처를 WgpuSpriteRenderer<SA>의 SA를 직접 사용하도록 변경
        // pub fn render(
        //     &mut self,
        //     surface_texture_view: &wgpu::TextureView,
        //     command_encoder: &mut wgpu::CommandEncoder,
        //     mut render_commands: Vec<SpriteRenderCommand<SA>>, // SA를 직접 사용
        // ) where
        //     SA: Ord, // WgpuSpriteRenderer<SA>에 Eq + Hash + Copy가 이미 있으므로 Ord만 추가
        // { ... }
        // 위와 같이 하려면 SpriteRenderCommand도 WgpuSpriteRenderer와 동일한 SA를 사용해야 합니다.
        // WgpuEngine2D에서 SpriteRenderCommand<SA>를 사용하므로 이 방식이 더 자연스러울 것입니다.
        // 아래 코드는 render_commands: Vec<SpriteRenderCommand<SA>> 이고, SA: Ord 제약이 있다고 가정하고 진행합니다.

        let device = match self.device.as_ref() {
            Some(d) => d,
            None => {
                println!(
                    "SpriteRenderer::render called before on_gpu_resources_ready or after on_gpu_resources_lost"
                );
                return;
            }
        };
        let queue = self.queue.as_ref().unwrap();
        let sampler = self.sampler.as_ref().unwrap();
        let sprite_bgl = self.sprite_bind_group_layout.as_ref().unwrap();
        let quad_vb = self.quad_vertex_buffer.as_ref().unwrap();
        let quad_ib = self.quad_index_buffer.as_ref().unwrap();
        let pipeline = self.pipeline.as_ref().unwrap();
        let screen_bind_group = self.screen_bind_group.as_ref().unwrap();

        if render_commands.is_empty() {
            command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("sprite clear pass"),
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
            return;
        }

        // 1. Render commands를 sprite_asset_id 기준으로 정렬
        render_commands.sort_unstable_by_key(|cmd| cmd.sprite_asset_id); // SA가 Ord를 구현해야 함

        // 2. InstanceData 생성
        let instance_data_vec: Vec<InstanceData> = render_commands
            .iter()
            .map(|cmd| InstanceData {
                offset: [cmd.x, cmd.y],
                size: [cmd.texture.width as f32, cmd.texture.height as f32],
            })
            .collect();

        // 3. Instance buffer 업데이트 또는 생성
        let num_instances = instance_data_vec.len();
        if num_instances > self.instance_buffer_capacity || self.instance_buffer.is_none() {
            if let Some(old_buffer) = self.instance_buffer.take() {
                old_buffer.destroy();
            }
            self.instance_buffer_capacity = num_instances.next_power_of_two();
            if self.instance_buffer_capacity == 0 {
                self.instance_buffer_capacity = 16;
            }

            let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("instance buffer"),
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

        // 4. Render pass 시작
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
        rpass.set_vertex_buffer(0, quad_vb.slice(..));
        rpass.set_vertex_buffer(
            1,
            current_instance_buffer.slice(
                ..(num_instances * std::mem::size_of::<InstanceData>()) as wgpu::BufferAddress,
            ),
        );
        rpass.set_index_buffer(quad_ib.slice(..), wgpu::IndexFormat::Uint16);
        rpass.set_bind_group(0, screen_bind_group, &[]);

        // 5. 배칭하여 그리기
        let mut current_batch_start_index: u32 = 0;
        // current_asset_id는 이제 Option<SA> 타입 (SA가 Copy이므로 Option으로 감싸도 문제 없음)
        let mut current_asset_id: Option<SA> = None;

        for (i, cmd) in render_commands.iter().enumerate() {
            let asset_id = cmd.sprite_asset_id; // SA는 Copy이므로 직접 사용

            if current_asset_id.is_none() {
                current_asset_id = Some(asset_id);
            } else if current_asset_id != Some(asset_id) {
                let prev_asset_id = current_asset_id.unwrap(); // 이전 ID는 반드시 존재
                let sprite_bind_group = self
                    .bind_group_cache
                    .get(&prev_asset_id)
                    .expect("Bind group should have been created for previous asset_id");

                rpass.set_bind_group(1, sprite_bind_group, &[]);
                rpass.draw_indexed(0..6, 0, current_batch_start_index..i as u32);

                current_asset_id = Some(asset_id);
                current_batch_start_index = i as u32;
            }

            // 현재 asset_id에 대한 바인드 그룹이 캐시에 없으면 생성
            // cmd.texture.view를 사용해야 하므로 cmd.texture는 여전히 필요.
            self.bind_group_cache.entry(asset_id).or_insert_with(|| {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("sprite bg"), // 디버깅을 위해 asset_id 정보 추가 가능
                    layout: sprite_bgl,
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
        }

        // 마지막 배치 그리기
        if let Some(asset_id) = current_asset_id {
            let sprite_bind_group = self
                .bind_group_cache
                .get(&asset_id)
                .expect("Bind group should exist for the last batch");
            rpass.set_bind_group(1, sprite_bind_group, &[]);
            rpass.draw_indexed(0..6, 0, current_batch_start_index..num_instances as u32);
        }
    }
}
