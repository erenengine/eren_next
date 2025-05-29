use eren_core::{context::GameContext, core::Updatable};

pub struct Sprite {}

impl Sprite {
    pub fn new(x: f32, y: f32, texture_id: &str) -> Self {
        Self {}
    }
}

impl Updatable for Sprite {
    fn update(&mut self, context: &mut GameContext) {}
}
