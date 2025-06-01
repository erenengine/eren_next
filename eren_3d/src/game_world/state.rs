use eren_core::game_world::state::{assets::AssetsState, input::InputState};
use glam::Mat4;
use winit::dpi::PhysicalSize;

pub struct RenderRequest<MA> {
    pub matrix: Mat4,
    pub alpha: f32,
    pub model_asset_id: MA,
}

pub struct GameState<MA> {
    pub delta_time: f32,
    pub model_assets: AssetsState<MA>,
    pub input: InputState,
    pub render_requests: Vec<RenderRequest<MA>>,
    pub window_size: PhysicalSize<u32>,
}

impl<MA> GameState<MA> {
    pub fn new() -> Self {
        Self {
            delta_time: 0.0,
            model_assets: AssetsState::new(),
            input: InputState::new(),
            render_requests: Vec::new(),
            window_size: PhysicalSize::new(0, 0),
        }
    }
}
