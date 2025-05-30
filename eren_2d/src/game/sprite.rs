use eren_core::{game::state::GameState, update::Update};

pub struct Sprite {}

impl Sprite {
    pub fn new(x: f32, y: f32, asset_id: String) -> Self {
        Self {}
    }
}

impl Update for Sprite {
    fn update(&mut self, state: &mut GameState) {}
}
