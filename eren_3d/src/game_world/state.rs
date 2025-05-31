use eren_core::game_world::state::input::InputState;

struct RenderRequest {
    //TODO
}

pub struct GameState {
    input: InputState,
    render_requests: Vec<RenderRequest>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            input: InputState::new(),
            render_requests: Vec::new(),
        }
    }
}
