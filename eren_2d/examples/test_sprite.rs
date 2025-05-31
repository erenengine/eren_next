use eren_2d::{
    app::{App, AppConfig},
    game_world::{sprite::Sprite, state::GameState, update::Update},
};
use eren_core::render_world::common::gpu::GraphicsLibrary;

#[derive(Clone, Copy)]
enum SpriteAssets {
    Logo,
    TestSprite,
}

struct Root {
    sprite: Sprite<SpriteAssets>,
}

impl Root {
    pub fn new() -> Self {
        Self {
            sprite: Sprite::new(0.0, 0.0, SpriteAssets::TestSprite),
        }
    }
}

impl Update<SpriteAssets> for Root {
    fn update(&mut self, state: &mut GameState<SpriteAssets>) {
        self.sprite.update(state);
    }
}

fn main() {
    App::new(
        AppConfig {
            window_width: 1280,
            window_height: 720,
            window_title: "Test Sprite".to_string(),
            graphics_library: GraphicsLibrary::Wgpu,
        },
        Root::new(),
    )
    .run();
}
