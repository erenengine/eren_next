use crate::render::gpu::GpuState;
use winit::window::Window;

pub struct WgpuGpuState {}

impl WgpuGpuState {
    pub fn new() -> Self {
        Self {}
    }
}

impl GpuState for WgpuGpuState {
    fn init(&mut self, window: &Window) {}

    fn cleanup(&mut self) {}

    fn resize_surface(&mut self, width: u32, height: u32) {}

    fn draw_frame(&mut self) {}
}
