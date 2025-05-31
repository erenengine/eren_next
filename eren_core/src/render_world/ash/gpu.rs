use std::sync::Arc;

use super::engine::AshEngine;
use crate::render_world::common::gpu::GpuResourceManager;
use winit::window::Window;

pub struct AshGpuResourceManager {
    engine: Box<dyn AshEngine>,
}

impl AshGpuResourceManager {
    pub fn new(engine: Box<dyn AshEngine>) -> Self {
        Self { engine }
    }
}

impl GpuResourceManager for AshGpuResourceManager {
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
