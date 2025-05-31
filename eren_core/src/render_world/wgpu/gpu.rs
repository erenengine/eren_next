use std::sync::Arc;

use super::engine::WgpuEngine;
use crate::render_world::common::gpu::GpuResourceManager;
use winit::window::Window;

pub struct WgpuGpuResourceManager {
    engine: Box<dyn WgpuEngine>,
}

impl WgpuGpuResourceManager {
    pub fn new(engine: Box<dyn WgpuEngine>) -> Self {
        Self { engine }
    }
}

impl GpuResourceManager for WgpuGpuResourceManager {
    fn on_window_ready(&mut self, window: Arc<Window>) {
        self.engine.on_gpu_resources_ready();
    }

    fn on_window_lost(&mut self) {
        self.engine.on_gpu_resources_lost();
    }

    fn on_window_resized(&mut self, width: u32, height: u32) {}

    fn update(&mut self) {
        self.engine.update();
    }
}
