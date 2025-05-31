use std::{collections::HashMap, hash::Hash};

use super::state::GameState;

pub struct AssetBundle<SA> {
    pending_sprite_assets: Vec<(SA, String)>,
}

impl<SA: Eq + Hash + Clone> AssetBundle<SA> {
    pub fn new(sprite_assets: Vec<(SA, String)>) -> Self {
        Self {
            pending_sprite_assets: sprite_assets,
        }
    }

    pub fn is_loaded(&mut self, game_state: &mut GameState<SA>) -> bool {
        if self.pending_sprite_assets.is_empty() {
            return true;
        }

        let ready = &game_state.sprite_assets.ready;
        let global_pending = &mut game_state.sprite_assets.pending;

        self.pending_sprite_assets.retain(|(asset, path)| {
            if ready.contains(asset) {
                global_pending.remove(asset);
                false
            } else {
                global_pending.entry(asset.clone()).or_insert(path.clone());
                true
            }
        });

        self.pending_sprite_assets.is_empty()
    }
}
