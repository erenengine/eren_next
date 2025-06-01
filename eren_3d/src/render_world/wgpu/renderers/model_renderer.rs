use crate::render_world::wgpu::{
    asset_managers::model_asset_manager::MeshGpuResource,
    model::{Mesh, Vertex},
};
use wgpu::util::DeviceExt;

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
    pub mesh: MeshGpuResource,
    pub material_asset_id: MA,
    pub bind_group: wgpu::BindGroup,
}

pub struct WgpuModelRenderer<MA> {
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,

    pipeline: Option<wgpu::RenderPipeline>,
    camera_buffer: Option<wgpu::Buffer>,
    camera_bind_group: Option<wgpu::BindGroup>,

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
        material_bind_group_layout: &wgpu::BindGroupLayout,
        camera_uniform: CameraUniform,
    ) {
        self.device = Some(device.clone());
        self.queue = Some(queue.clone());

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
            source: wgpu::ShaderSource::Wgsl(include_str!("model.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("model pipeline layout"),
            bind_group_layouts: &[&camera_bind_group_layout, &material_bind_group_layout],
            push_constant_ranges: &[],
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
            depth_stencil: None, // 필요하면 추가
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        self.pipeline = Some(pipeline);
        self.camera_buffer = Some(camera_buffer);
        self.camera_bind_group = Some(camera_bind_group);
    }

    pub fn on_gpu_resources_lost(&mut self) {
        self.device = None;
        self.queue = None;

        self.pipeline = None;
        self.camera_buffer = None;
        self.camera_bind_group = None;

        self.instance_buffer = None;
        self.instance_buffer_capacity = 0;
    }

    pub fn render(
        &mut self,
        surface_texture_view: &wgpu::TextureView,
        command_encoder: &mut wgpu::CommandEncoder,
        render_commands: Vec<ModelRenderCommand<MA>>,
    ) {
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

            let mut render_pass = command_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("model render pass"),
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
                depth_stencil_attachment: None, // 필요하면 depth texture 추가
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(pipeline);
            render_pass.set_vertex_buffer(1, current_instance_buffer.slice(..));
            render_pass.set_bind_group(0, camera_bind_group, &[]);

            let mut current_asset_id: Option<MA> = None;
            let mut batch_start = 0u32;

            for (i, cmd) in render_commands.iter().enumerate() {
                render_pass.set_vertex_buffer(0, cmd.mesh.vertex_buffer.slice(..));
                render_pass
                    .set_index_buffer(cmd.mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

                if current_asset_id != Some(cmd.material_asset_id) {
                    if i as u32 > batch_start {
                        render_pass.draw_indexed(0..cmd.mesh.num_indices, 0, batch_start..i as u32);
                    }
                    render_pass.set_bind_group(1, &cmd.bind_group, &[]);
                    current_asset_id = Some(cmd.material_asset_id);
                    batch_start = i as u32;
                }
            }

            if render_commands.len() as u32 > batch_start {
                let last_cmd = &render_commands[render_commands.len() - 1];
                render_pass.draw_indexed(
                    0..last_cmd.mesh.num_indices,
                    0,
                    batch_start..render_commands.len() as u32,
                );
            }
        }
    }
}
