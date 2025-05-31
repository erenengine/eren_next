use eren_core::{
    game_world::state::{assets::AssetsState, input::InputState},
    math::Vec2,
};

struct RenderRequest<SA> {
    position: Vec2,
    sprite_asset_id: SA,
}

pub struct GameState<SA> {
    assets: AssetsState,
    input: InputState,
    render_requests: Vec<RenderRequest<SA>>,
}

impl<SA> GameState<SA> {
    pub fn new() -> Self {
        Self {
            assets: AssetsState::new(),
            input: InputState::new(),
            render_requests: Vec::new(),
        }
    }
}
