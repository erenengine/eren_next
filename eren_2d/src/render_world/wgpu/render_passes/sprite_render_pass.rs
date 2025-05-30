use crate::game_world::state::GameState;

pub struct WgpuSpriteRenderPass<AssetId> {
    bind_group_layout: Option<wgpu::BindGroupLayout>,
    phantom: std::marker::PhantomData<AssetId>,
}

impl<AssetId> WgpuSpriteRenderPass<AssetId> {
    pub fn new() -> Self {
        Self {
            bind_group_layout: None,
            phantom: std::marker::PhantomData,
        }
    }

    fn create_gpu_resources(&mut self, device: &wgpu::Device) {
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

    fn destroy_gpu_resources(&mut self) {
        self.bind_group_layout = None;
    }

    fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        state: &GameState<AssetId>,
    ) {
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
