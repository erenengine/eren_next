use super::{state::GameState, transform::GlobalTransform};

pub trait Update<SA> {
    fn update(&mut self, game_state: &mut GameState<SA>, parent_global_transform: &GlobalTransform);
}
