use super::state::GameState;

pub trait Update<SA> {
    fn update(&mut self, state: &mut GameState<SA>);
}
