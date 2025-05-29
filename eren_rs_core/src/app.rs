use crate::{context::GameContext, core::Updatable};

pub struct App<T: Updatable> {
    context: GameContext,
    pub root: T,
}

impl<T: Updatable> App<T> {
    pub fn new(root: T) -> Self {
        Self {
            context: GameContext::new(),
            root,
        }
    }
    pub fn run(&mut self) {
        self.root.update(&mut self.context);
    }
}
