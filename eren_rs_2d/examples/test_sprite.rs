use eren_2d::sprite::Sprite;
use eren_core::{app::App, asset::AssetBundle, context::GameContext, core::Updatable};

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

impl Updatable for Root {
    fn update(&mut self, context: &mut GameContext) {
        if self.in_game_screen.is_asset_loaded() {
            self.loading_screen = None;
        }

        if let Some(screen) = self.loading_screen.as_mut() {
            screen.update(context);
        } else {
            self.in_game_screen.update(context);
        }
    }
}

struct LoadingScreen {
    asset_bundle: AssetBundle,
}

impl LoadingScreen {
    pub fn new() -> Self {
        Self {
            asset_bundle: AssetBundle::new(vec![("logo", "examples/assets/logo.png")]),
        }
    }
}

impl Updatable for LoadingScreen {
    fn update(&mut self, context: &mut GameContext) {}
}

struct InGameScreen {
    asset_bundle: AssetBundle,
    sprite: Sprite,
}

impl InGameScreen {
    pub fn new() -> Self {
        Self {
            asset_bundle: AssetBundle::new(vec![(
                "test_sprite",
                "examples/assets/test_sprite.png",
            )]),
            sprite: Sprite::new(0.0, 0.0, "test_sprite"),
        }
    }

    pub fn is_asset_loaded(&self) -> bool {
        self.asset_bundle.is_loaded()
    }
}

impl Updatable for InGameScreen {
    fn update(&mut self, context: &mut GameContext) {
        if self.asset_bundle.is_loaded() {
            self.sprite.update(context);
        }
    }
}

fn main() {
    App::new(Root::new()).run();
}
