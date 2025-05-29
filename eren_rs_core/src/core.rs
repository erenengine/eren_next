use crate::context::GameContext;

pub trait Updatable {
    fn update(&mut self, context: &mut GameContext);
}
