pub trait WgpuAssetLoader {
    fn load(&mut self, path: String);
    fn upload_textures(&mut self, device: &wgpu::Device, queue: &wgpu::Queue);
    fn unload_textures(&mut self);
}
