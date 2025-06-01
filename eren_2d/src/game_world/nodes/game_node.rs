use crate::game_world::{state::GameState, transform::GlobalTransform};

pub trait GameNode<SA> {
    fn update(&mut self, game_state: &mut GameState<SA>, parent_global_transform: &GlobalTransform);
}
