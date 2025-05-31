use winit::dpi::PhysicalSize;

pub trait AshEngine {
    fn on_gpu_resources_ready(&mut self);

    fn on_gpu_resources_lost(&mut self);

    fn on_window_resized(&mut self, window_size: PhysicalSize<u32>);

    fn update(&mut self);
}
