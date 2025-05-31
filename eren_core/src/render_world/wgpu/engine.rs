pub trait WgpuEngine {
    fn on_gpu_resources_ready(&mut self);
    fn on_gpu_resources_lost(&mut self);
    fn update(&mut self);
}
