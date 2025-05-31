pub struct WgpuSpriteAssetManager {}

impl WgpuSpriteAssetManager {
    pub fn new() -> Self {
        Self {}
    }

    pub fn on_gpu_resources_ready(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {}

    pub fn on_gpu_resources_lost(&mut self) {}
}
