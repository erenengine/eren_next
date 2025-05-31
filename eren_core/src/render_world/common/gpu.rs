use std::sync::Arc;

use winit::window::Window;

#[derive(PartialEq)]
pub enum GraphicsLibrary {
    Ash,
    Wgpu,
}

pub trait GpuResourceManager {
    fn on_window_ready(&mut self, window: Arc<Window>);

    fn on_window_lost(&mut self);

    fn on_window_resized(&mut self, width: u32, height: u32);

    fn update(&mut self);
}
