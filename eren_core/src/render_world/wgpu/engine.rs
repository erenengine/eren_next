pub trait WgpuEngine {
    fn on_gpu_resources_ready(&mut self, device: &wgpu::Device, queue: &wgpu::Queue);

    fn on_gpu_resources_lost(&mut self);

    fn update(
        &mut self,
        surface_texture_view: &wgpu::TextureView,
        command_encoder: &mut wgpu::CommandEncoder,
    );
}
