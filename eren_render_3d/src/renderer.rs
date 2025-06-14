use eren_render_core::renderer::{FrameContext, Renderer};
use eren_window::window::WindowSize;

use crate::passes::test_pass::TestPass;

pub struct Renderer3D {
    test_pass: TestPass,
}

impl Renderer3D {
    pub fn new(
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        window_size: WindowSize,
    ) -> Self {
        Self {
            test_pass: TestPass::new(device, surface_format, window_size),
        }
    }

    pub fn on_window_resized(&mut self, queue: &wgpu::Queue, window_size: WindowSize) {
        self.test_pass.update_quad_size_buffer(queue, window_size);
    }
}

impl Renderer for Renderer3D {
    fn render<'a>(&self, frame_context: &mut FrameContext<'a>) {
        self.test_pass.draw_frame(frame_context);
    }
}
