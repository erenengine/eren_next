use std::sync::Arc;

use winit::{dpi::PhysicalSize, window::Window};

#[derive(PartialEq)]
pub enum GraphicsLibrary {
    Ash,
    Wgpu,
}

pub trait GpuResourceManager {
    fn on_window_ready(&mut self, window: Arc<Window>);

    fn on_window_lost(&mut self);

    fn on_window_resized(&mut self, window_size: PhysicalSize<u32>);

    fn update(&mut self);
}
