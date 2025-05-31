use winit::dpi::PhysicalSize;

pub trait WgpuEngine {
    fn on_gpu_resources_ready(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        window_size: PhysicalSize<u32>,
        window_scale_factor: f64,
    );

    fn on_gpu_resources_lost(&mut self);

    fn on_window_resized(&mut self, window_size: PhysicalSize<u32>, window_scale_factor: f64);

    fn update(
        &mut self,
        surface_texture_view: &wgpu::TextureView,
        command_encoder: &mut wgpu::CommandEncoder,
    );
}
