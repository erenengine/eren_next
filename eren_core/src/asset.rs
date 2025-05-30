use crate::game::state::GameState;

pub trait AssetManager {
    fn ensure_asset_loaded(&mut self, state: &mut GameState);
}

pub struct AssetBundle {
    pending_assets: Vec<(String, String)>,
}

impl AssetBundle {
    pub fn new(assets: Vec<(String, String)>) -> Self {
        Self {
            pending_assets: assets,
        }
    }

    pub fn is_loaded(&mut self, state: &mut GameState) -> bool {
        self.pending_assets
            .retain(|(asset_id, _)| !state.is_asset_loaded(asset_id.clone()));
        self.pending_assets.is_empty()
    }
}
