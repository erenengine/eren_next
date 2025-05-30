use eren_core::render::wgpu::pass::WgpuRenderPass;

pub struct WgpuSpriteVertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
    pub offset: [f32; 2],
    pub scale: [f32; 2],
}

impl WgpuSpriteVertex {
    pub const ATTRIBS: [wgpu::VertexAttribute; 4] = wgpu::vertex_attr_array![
        0 => Float32x2, // pos
        1 => Float32x2, // uv
        2 => Float32x2, // offset
        3 => Float32x2, // scale
    ];

    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<WgpuSpriteVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub struct WgpuSpriteRenderPass {
    bind_group_layout: Option<wgpu::BindGroupLayout>,
}

impl WgpuSpriteRenderPass {
    pub fn new() -> Self {
        Self {
            bind_group_layout: None,
        }
    }
}

impl WgpuRenderPass for WgpuSpriteRenderPass {
    fn surface_created(&mut self, device: &wgpu::Device) {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sprite_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
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

        self.bind_group_layout = Some(bind_group_layout);
    }

    fn surface_destroyed(&mut self) {}

    fn window_resized(&mut self) {}

    fn render(&mut self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let _rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("sprite_render_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
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
    }
}
