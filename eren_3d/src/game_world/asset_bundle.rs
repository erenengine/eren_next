use std::hash::Hash;

use super::state::GameState;

pub struct AssetBundle<MA> {
    pending_model_assets: Vec<(MA, String)>,
}

impl<MA: Eq + Hash + Clone> AssetBundle<MA> {
    pub fn new(model_assets: Vec<(MA, String)>) -> Self {
        Self {
            pending_model_assets: model_assets,
        }
    }

    pub fn is_loaded(&mut self, game_state: &mut GameState<MA>) -> bool {
        if self.pending_model_assets.is_empty() {
            return true;
        }

        let ready = &game_state.model_assets.ready;
        let global_pending = &mut game_state.model_assets.pending;

        self.pending_model_assets.retain(|(asset, path)| {
            if ready.contains(asset) {
                global_pending.remove(asset);
                false
            } else {
                global_pending.entry(asset.clone()).or_insert(path.clone());
                true
            }
        });

        self.pending_model_assets.is_empty()
    }
}
