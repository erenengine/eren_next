use crate::{asset::AssetManager, context::GameContext, core::Updatable};

pub struct App<T: Updatable> {
    context: GameContext,
    pub asset_manager: AssetManager,
    pub root: T,
}

impl<T: Updatable> App<T> {
    pub fn new(root: T) -> Self {
        Self {
            context: GameContext::new(),
            asset_manager: AssetManager::new(),
            root,
        }
    }

    pub fn run(&mut self) {
        self.root.update(&mut self.context);
    }
}
