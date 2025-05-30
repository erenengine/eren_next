pub struct RenderContext {}

pub trait RenderPassHandler {
    fn render_pass(&mut self, context: &mut RenderContext);
}
