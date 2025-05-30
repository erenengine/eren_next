pub struct AshRenderContext {}

pub trait AshRenderPass {
    fn app_resumed(&mut self, context: &mut AshRenderContext);
    fn app_suspended(&mut self, context: &mut AshRenderContext);
    fn window_resized(&mut self, context: &mut AshRenderContext);
    fn render(&mut self, context: &mut AshRenderContext);
}
