use eren_2d::{
    app::{App, AppConfig},
    game_world::{state::GameState, update::Update},
};
use eren_core::render_world::common::gpu::GraphicsLibrary;

enum SpriteAssets {
    Logo,
    TestSprite,
}

struct Root {}

impl Root {
    pub fn new() -> Self {
        Self {}
    }
}

impl Update<SpriteAssets> for Root {
    fn update(&mut self, state: &mut GameState<SpriteAssets>) {}
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
