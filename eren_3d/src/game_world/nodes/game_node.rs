use crate::game_world::{state::GameState, transform::GlobalTransform};

pub trait GameNode<MA> {
    fn update(&mut self, game_state: &mut GameState<MA>, parent_global_transform: &GlobalTransform);
}
