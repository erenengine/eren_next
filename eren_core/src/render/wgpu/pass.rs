pub trait WgpuRenderPass {
    fn surface_created(&mut self, device: &wgpu::Device);
    fn surface_destroyed(&mut self);
    fn window_resized(&mut self);
    fn render(&mut self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView);
}
