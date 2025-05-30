use eren_core::render::wgpu::pass::WgpuRenderPass;

pub struct WgpuSpriteRenderPass {}

impl WgpuSpriteRenderPass {
    pub fn new() -> Self {
        Self {}
    }
}

impl WgpuRenderPass for WgpuSpriteRenderPass {
    fn surface_created(&mut self) {}

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
