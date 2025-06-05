use std::hash::Hash;

use super::state::GameState;

pub struct AssetBundle<SA> {
    pending_sprite_assets: Vec<(SA, &'static str)>,
}

impl<SA: Eq + Hash + Clone> AssetBundle<SA> {
    pub fn new(sprite_assets: Vec<(SA, &'static str)>) -> Self {
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
                if !global_pending.contains_key(asset) {
                    global_pending.insert(asset.clone(), path);
                }
                true
            }
        });

        if self.pending_sprite_assets.is_empty() {
            self.pending_sprite_assets.shrink_to_fit(); // drop memory
            return true;
        }

        false
    }
}
