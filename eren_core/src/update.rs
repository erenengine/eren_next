use crate::game::context::GameContext;

pub trait Update {
    fn update(&mut self, state: &mut GameContext);
}
