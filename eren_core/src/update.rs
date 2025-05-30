use crate::game::state::GameState;

pub trait Update {
    fn update(&mut self, state: &mut GameState);
}
