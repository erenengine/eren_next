use std::sync::Arc;

use crate::{game::state::GameState, render::gpu::GpuContext};
use winit::window::Window;

use super::pass::AshRenderPass;

pub struct AshGpuContext {
    render_passes: Vec<Box<dyn AshRenderPass>>,
}

impl AshGpuContext {
    pub fn new() -> Self {
        Self {
            render_passes: Vec::new(),
        }
    }

    pub fn add_render_pass(&mut self, render_pass: Box<dyn AshRenderPass>) {
        self.render_passes.push(render_pass);
    }
}

impl GpuContext for AshGpuContext {
    fn create_surface(&mut self, window: Arc<Window>) {}

    fn destroy_surface(&mut self) {}

    fn resize_surface(&mut self, width: u32, height: u32) {}

    fn update(&mut self, state: &mut GameState) {}
}
