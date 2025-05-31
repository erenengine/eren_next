use super::state::GameState;

pub trait Update<SA> {
    fn update(&mut self, game_state: &mut GameState<SA>);
}
