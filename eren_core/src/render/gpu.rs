use std::sync::Arc;

use winit::window::Window;

use crate::game::state::GameState;

pub trait GpuContext {
    fn create_surface(&mut self, window: Arc<Window>);
    fn destroy_surface(&mut self);
    fn resize_surface(&mut self, width: u32, height: u32);
    fn update(&mut self, state: &mut GameState);
}
