#[derive(Debug)]
pub struct FrameContext<'a> {
    pub view: &'a wgpu::TextureView,
    pub encoder: &'a mut wgpu::CommandEncoder,
}

pub trait Renderer {
    fn render<'a>(&self, frame_context: &mut FrameContext<'a>);
}
