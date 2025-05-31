pub struct SpriteRenderCommand {}

pub struct WgpuSpriteRenderer {}

impl WgpuSpriteRenderer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn on_gpu_resources_ready(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {}

    pub fn on_gpu_resources_lost(&mut self) {}

    pub fn render(&mut self, render_commands: Vec<SpriteRenderCommand>) {}
}
