use ash::vk;

#[derive(Debug)]
pub struct FrameContext {
    pub command_buffer: vk::CommandBuffer,
    pub image_index: usize,
}

pub trait Renderer<R> {
    fn render(&self, frame_context: &FrameContext, render_items: &[R]);
}
