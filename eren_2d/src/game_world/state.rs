use eren_core::game_world::state::{assets::AssetsState, input::InputState};

pub struct RenderRequest<SA> {
    pub x: f32,
    pub y: f32,
    pub sprite_asset_id: SA,
}

pub struct GameState<SA> {
    sprite_assets: AssetsState<SA>,
    input: InputState,
    pub render_requests: Vec<RenderRequest<SA>>,
}

impl<SA> GameState<SA> {
    pub fn new() -> Self {
        Self {
            sprite_assets: AssetsState::new(),
            input: InputState::new(),
            render_requests: Vec::new(),
        }
    }
}
