use super::state::GameState;

pub trait Update<AssetId> {
    fn update(&mut self, state: &mut GameState<AssetId>);
}
