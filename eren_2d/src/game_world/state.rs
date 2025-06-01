use eren_core::game_world::state::{assets::AssetsState, input::InputState};
use glam::Vec2;
use winit::dpi::PhysicalSize;

pub struct RenderRequest<SA> {
    pub position: Vec2,
    pub scale: Vec2,
    pub rotation: f32,
    pub alpha: f32,
    pub sprite_asset_id: SA,
}

pub struct GameState<SA> {
    pub delta_time: f32,
    pub sprite_assets: AssetsState<SA>,
    pub input: InputState,
    pub render_requests: Vec<RenderRequest<SA>>,
    pub window_size: PhysicalSize<u32>,
}

impl<SA> GameState<SA> {
    pub fn new() -> Self {
        Self {
            delta_time: 0.0,
            sprite_assets: AssetsState::new(),
            input: InputState::new(),
            render_requests: Vec::new(),
            window_size: PhysicalSize::new(0, 0),
        }
    }
}
