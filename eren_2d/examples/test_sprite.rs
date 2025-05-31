use eren_2d::{
    app::{App, AppConfig},
    game_world::{asset_bundle::AssetBundle, sprite::Sprite, state::GameState, update::Update},
};
use eren_core::render_world::common::gpu::GraphicsLibrary;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum SpriteAssets {
    Logo,
    TestSprite,
}

struct Root {
    loading_screen: Option<LoadingScreen>,
    in_game_screen: InGameScreen,
}

impl Root {
    pub fn new() -> Self {
        Self {
            loading_screen: Some(LoadingScreen::new()),
            in_game_screen: InGameScreen::new(),
        }
    }
}

impl Update<SpriteAssets> for Root {
    fn update(&mut self, game_state: &mut GameState<SpriteAssets>) {
        if self.in_game_screen.is_asset_loaded(game_state) {
            self.loading_screen = None;
        }

        if let Some(loading_screen) = self.loading_screen.as_mut() {
            loading_screen.update(game_state);
        } else {
            self.in_game_screen.update(game_state);
        }
    }
}

struct LoadingScreen {
    asset_bundle: AssetBundle<SpriteAssets>,
}

impl LoadingScreen {
    pub fn new() -> Self {
        Self {
            asset_bundle: AssetBundle::new(vec![(
                SpriteAssets::Logo,
                "examples/assets/logo.png".into(),
            )]),
        }
    }
}

impl Update<SpriteAssets> for LoadingScreen {
    fn update(&mut self, _state: &mut GameState<SpriteAssets>) {}
}

struct InGameScreen {
    asset_bundle: AssetBundle<SpriteAssets>,
    sprite1: Sprite<SpriteAssets>,
    sprite2: Sprite<SpriteAssets>,
}

impl InGameScreen {
    pub fn new() -> Self {
        Self {
            asset_bundle: AssetBundle::new(vec![(
                SpriteAssets::TestSprite,
                "examples/assets/test_sprite.png".into(),
            )]),
            sprite1: Sprite::new(100.0, 100.0, SpriteAssets::TestSprite),
            sprite2: Sprite::new(-100.0, -100.0, SpriteAssets::TestSprite),
        }
    }

    pub fn is_asset_loaded(&mut self, state: &mut GameState<SpriteAssets>) -> bool {
        self.asset_bundle.is_loaded(state)
    }
}

impl Update<SpriteAssets> for InGameScreen {
    fn update(&mut self, state: &mut GameState<SpriteAssets>) {
        if self.asset_bundle.is_loaded(state) {
            self.sprite1.update(state);
            self.sprite2.update(state);
        }
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
