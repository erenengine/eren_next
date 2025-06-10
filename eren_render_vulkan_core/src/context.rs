use eren_window::window::WindowSize;
use winit::window::Window;

#[derive(Debug)]
pub struct FrameContext {}

pub struct GraphicsContext<F>
where
    F: Fn(&FrameContext),
{
    draw_frame: F,
}

impl<F> GraphicsContext<F>
where
    F: Fn(&FrameContext),
{
    pub fn new(draw_frame: F) -> Self {
        Self { draw_frame }
    }

    pub fn init(&mut self, window: &Window) {}

    pub fn resize(&mut self, window_size: WindowSize) {}

    pub fn destroy(&mut self) {}

    pub fn redraw(&mut self) {}
}
