use std::sync::Arc;

use super::engine::AshEngine;
use crate::render_world::common::gpu::GpuResourceManager;
use ash::Entry;
use winit::{dpi::PhysicalSize, window::Window};

pub struct AshGpuResourceManager {
    engine: Box<dyn AshEngine>,
    entry: Entry,
}

impl AshGpuResourceManager {
    pub fn new(engine: Box<dyn AshEngine>) -> Self {
        let entry = unsafe { Entry::load().expect("Failed to load entry point") };
        Self { engine, entry }
    }
}

impl GpuResourceManager for AshGpuResourceManager {
    fn on_window_ready(&mut self, window: Arc<Window>) {
        self.engine.on_gpu_resources_ready();
    }

    fn on_window_lost(&mut self) {
        self.engine.on_gpu_resources_lost();
    }

    fn on_window_resized(&mut self, window_size: PhysicalSize<u32>, window_scale_factor: f64) {
        self.engine
            .on_window_resized(window_size, window_scale_factor);
    }

    fn update(&mut self) {
        self.engine.update();
    }
}
