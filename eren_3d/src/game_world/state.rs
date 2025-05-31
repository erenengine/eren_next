use eren_core::game_world::state::{assets::AssetsState, input::InputState};

struct RenderRequest {
    //TODO
}

pub struct GameState {
    assets: AssetsState,
    input: InputState,
    render_requests: Vec<RenderRequest>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            assets: AssetsState::new(),
            input: InputState::new(),
            render_requests: Vec::new(),
        }
    }
}
