use ash::vk;

#[derive(Debug)]
pub struct FrameContext {
    pub command_buffer: vk::CommandBuffer,
    pub image_index: usize,
}

pub trait Renderer {
    fn render(&self, frame_context: &FrameContext);
}
