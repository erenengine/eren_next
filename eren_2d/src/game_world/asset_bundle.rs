use super::state::GameState;

pub struct AssetBundle<SA> {
    pending_sprite_assets: Vec<(SA, String)>,
}

impl<SA> AssetBundle<SA> {
    pub fn new(sprite_assets: Vec<(SA, String)>) -> Self {
        Self {
            pending_sprite_assets: sprite_assets,
        }
    }

    pub fn is_loaded(&mut self, state: &mut GameState<SA>) -> bool {
        //TODO
        false
    }
}
