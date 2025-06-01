use crate::render_world::wgpu::{
    asset_managers::model_asset_manager::ModelGpuResource, model::Vertex,
};
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;

const MODEL_SHADER: &str = include_str!("model.wgsl");

const BASE_COLOR: wgpu::Color = wgpu::Color {
    r: 0.1,
    g: 0.2,
    b: 0.3,
    a: 1.0,
};

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    pub view_proj: [[f32; 4]; 4], // mat4x4<f32>
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceData {
    matrix: [[f32; 4]; 4], // Model matrix (mat4)
    alpha: f32,
    _padding: [f32; 3],
}

impl InstanceData {
    const ATTRIBUTES: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
        3 => Float32x4, // matrix.col0
        4 => Float32x4, // matrix.col1
        5 => Float32x4, // matrix.col2
        6 => Float32x4, // matrix.col3
        7 => Float32,   // alpha
    ];
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<InstanceData>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

pub struct ModelRenderCommand<MA> {
    pub matrix: glam::Mat4,
    pub alpha: f32,
    pub material_asset_id: MA,
    pub model_gpu_resource: ModelGpuResource,
}

pub struct WgpuModelRenderer<MA> {
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,

    pipeline: Option<wgpu::RenderPipeline>,
    camera_buffer: Option<wgpu::Buffer>,
    camera_bind_group: Option<wgpu::BindGroup>,
    depth_view: Option<wgpu::TextureView>,

    instance_buffer: Option<wgpu::Buffer>,
    instance_buffer_capacity: usize,

    phantom: std::marker::PhantomData<MA>,
}

impl<MA: PartialEq + Copy> WgpuModelRenderer<MA> {
    pub fn new() -> Self {
        Self {
            device: None,
            queue: None,

            pipeline: None,
            camera_buffer: None,
            camera_bind_group: None,
            depth_view: None,

            instance_buffer: None,
            instance_buffer_capacity: 0,

            phantom: std::marker::PhantomData,
        }
    }

    fn create_camera_uniform(
        &self,
        window_size: PhysicalSize<u32>,
        window_scale_factor: f64,
    ) -> CameraUniform {
        let width = (window_size.width as f64 * window_scale_factor) as u32;
        let height = (window_size.height as f64 * window_scale_factor) as u32;

        let aspect = width as f32 / height as f32;

        let view = glam::Mat4::look_at_rh(
            glam::Vec3::new(0.0, 2.0, 5.0),
            glam::Vec3::ZERO,
            glam::Vec3::Y,
        );

        let proj = glam::Mat4::perspective_rh_gl(45.0_f32.to_radians(), aspect, 0.1, 100.0);

        CameraUniform {
            view_proj: (proj * view).to_cols_array_2d(),
        }
    }

    fn create_depth_view(
        &self,
        device: &wgpu::Device,
        window_size: PhysicalSize<u32>,
    ) -> wgpu::TextureView {
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth texture"),
            size: wgpu::Extent3d {
                width: window_size.width,
                height: window_size.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        depth_texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn on_gpu_resources_ready(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        material_bind_group_layout: &wgpu::BindGroupLayout,
        window_size: PhysicalSize<u32>,
        window_scale_factor: f64,
    ) {
        self.device = Some(device.clone());
        self.queue = Some(queue.clone());

        let camera_uniform = self.create_camera_uniform(window_size, window_scale_factor);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("camera bind group layout"),
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

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera bind group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("model shader"),
            source: wgpu::ShaderSource::Wgsl(MODEL_SHADER.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("model pipeline layout"),
            bind_group_layouts: &[&camera_bind_group_layout, &material_bind_group_layout],
            push_constant_ranges: &[],
        });

        let depth_stencil = Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth24PlusStencil8,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("model pipeline"),
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
            depth_stencil,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        self.pipeline = Some(pipeline);
        self.camera_buffer = Some(camera_buffer);
        self.camera_bind_group = Some(camera_bind_group);
        self.depth_view = Some(self.create_depth_view(device, window_size));
    }

    pub fn on_gpu_resources_lost(&mut self) {
        self.device = None;
        self.queue = None;

        self.pipeline = None;
        self.camera_buffer = None;
        self.camera_bind_group = None;
        self.depth_view = None;

        self.instance_buffer = None;
        self.instance_buffer_capacity = 0;
    }

    pub fn on_window_resized(&mut self, window_size: PhysicalSize<u32>, window_scale_factor: f64) {
        if let (Some(device), Some(buffer), Some(queue)) =
            (&self.device, &self.camera_buffer, &self.queue)
        {
            self.depth_view = Some(self.create_depth_view(device, window_size));

            let camera_uniform = self.create_camera_uniform(window_size, window_scale_factor);
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&[camera_uniform]));
        }
    }

    pub fn render(
        &mut self,
        surface_texture_view: &wgpu::TextureView,
        command_encoder: &mut wgpu::CommandEncoder,
        render_commands: Vec<ModelRenderCommand<MA>>,
    ) {
        let depth_stencil_attachment = if let Some(depth_view) = &self.depth_view {
            Some(wgpu::RenderPassDepthStencilAttachment {
                view: depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: None,
            })
        } else {
            None
        };

        let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("model render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: surface_texture_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(BASE_COLOR),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        if render_commands.is_empty() {
            return;
        }

        if let (Some(device), Some(queue), Some(pipeline), Some(camera_bind_group)) = (
            &self.device,
            &self.queue,
            &self.pipeline,
            &self.camera_bind_group,
        ) {
            let instance_data_vec: Vec<InstanceData> = render_commands
                .iter()
                .map(|cmd| InstanceData {
                    matrix: cmd.matrix.to_cols_array_2d(),
                    alpha: cmd.alpha,
                    _padding: [0.0; 3],
                })
                .collect();

            let num_instances = instance_data_vec.len();
            if num_instances > self.instance_buffer_capacity || self.instance_buffer.is_none() {
                if let Some(old_buffer) = self.instance_buffer.take() {
                    old_buffer.destroy();
                }

                self.instance_buffer_capacity = num_instances.next_power_of_two().max(16);

                let new_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("model instance buffer"),
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
            render_pass.set_vertex_buffer(1, current_instance_buffer.slice(..));
            render_pass.set_bind_group(0, camera_bind_group, &[]);

            for (instance_idx, cmd) in render_commands.iter().enumerate() {
                let instance_range = instance_idx as u32..instance_idx as u32 + 1;

                for mesh in &cmd.model_gpu_resource.meshes {
                    if let Some(bind_group) = &mesh.bind_group {
                        render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                        render_pass.set_index_buffer(
                            mesh.index_buffer.slice(..),
                            wgpu::IndexFormat::Uint32,
                        );
                        render_pass.set_bind_group(1, bind_group, &[]);
                        render_pass.draw_indexed(0..mesh.num_indices, 0, instance_range.clone());
                    }
                }
            }
        }
    }
}
