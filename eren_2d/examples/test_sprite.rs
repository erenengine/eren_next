use eren_2d::game_world::{sprite::Sprite, state::GameState};

enum Assets {
    Logo,
    TestSprite,
}

struct InGameScreen {
    sprite: Sprite<Assets>,
}

impl InGameScreen {
    pub fn new() -> Self {
        Self {
            sprite: Sprite::new(0.0, 0.0, Assets::TestSprite),
        }
    }
}

fn main() {}
