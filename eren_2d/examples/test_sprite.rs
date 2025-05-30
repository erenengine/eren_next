use eren_2d::{
    app::{App, AppConfig},
    game::sprite::Sprite,
};
use eren_core::{
    asset::AssetBundle, game::state::GameState, render::GraphicsLibrary, update::Update,
};

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

impl Update for Root {
    fn update(&mut self, state: &mut GameState) {
        if self.in_game_screen.is_asset_loaded(state) {
            self.loading_screen = None;
        }

        if let Some(loading_screen) = self.loading_screen.as_mut() {
            loading_screen.update(state);
        } else {
            self.in_game_screen.update(state);
        }
    }
}

struct LoadingScreen {
    asset_bundle: AssetBundle,
}

impl LoadingScreen {
    pub fn new() -> Self {
        Self {
            asset_bundle: AssetBundle::new(vec![(
                "logo".into(),
                "examples/assets/logo.png".into(),
            )]),
        }
    }
}

impl Update for LoadingScreen {
    fn update(&mut self, state: &mut GameState) {}
}

struct InGameScreen {
    asset_bundle: AssetBundle,
    sprite: Sprite,
}

impl InGameScreen {
    pub fn new() -> Self {
        Self {
            asset_bundle: AssetBundle::new(vec![(
                "test_sprite".into(),
                "examples/assets/test_sprite.png".into(),
            )]),
            sprite: Sprite::new(0.0, 0.0, "test_sprite".into()),
        }
    }

    pub fn is_asset_loaded(&mut self, state: &mut GameState) -> bool {
        self.asset_bundle.is_loaded(state)
    }
}

impl Update for InGameScreen {
    fn update(&mut self, state: &mut GameState) {
        if self.asset_bundle.is_loaded(state) {
            self.sprite.update(state);
        }
    }
}

fn main() {
    App::new(
        AppConfig {
            window_width: 800,
            window_height: 600,
            window_title: "Test Sprite".to_string(),
            graphics_library: GraphicsLibrary::Wgpu,
        },
        Root::new(),
    )
    .run();
}
