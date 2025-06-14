use eren_render_core::renderer::{FrameContext, Renderer};

use crate::passes::test_pass::TestPass;

pub struct Renderer3D {
    test_pass: TestPass,
}

impl Renderer3D {
    pub fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        Self {
            test_pass: TestPass::new(device, surface_format),
        }
    }
}

impl Renderer for Renderer3D {
    fn render<'a>(&self, frame_context: &mut FrameContext<'a>) {
        self.test_pass.draw_frame(frame_context);
    }
}
