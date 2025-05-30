use winit::window::Window;

pub trait GpuState {
    fn init(&mut self, window: &Window);
    fn cleanup(&mut self);
    fn resize_surface(&mut self, width: u32, height: u32);
    fn draw_frame(&mut self);
}
