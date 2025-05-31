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

struct Vec2 {
    x: f32,
    y: f32,
}

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

struct InGameScreen {
    asset_bundle: AssetBundle<SpriteAssets>,
    sprite1: Sprite<SpriteAssets>,
    sprite2: Sprite<SpriteAssets>,
    velocity1: Vec2,
    velocity2: Vec2,
}

impl InGameScreen {
    pub fn new() -> Self {
        Self {
            asset_bundle: AssetBundle::new(vec![(
                SpriteAssets::TestSprite,
                "examples/assets/test_sprite.png".into(),
            )]),
            sprite1: Sprite::new(100.0, 100.0, SpriteAssets::TestSprite),
            sprite2: Sprite::new(500.0, 400.0, SpriteAssets::TestSprite),
            velocity1: Vec2::new(1000.0, 1500.0),
            velocity2: Vec2::new(-1200.0, -800.0),
        }
    }

    pub fn is_asset_loaded(&mut self, state: &mut GameState<SpriteAssets>) -> bool {
        self.asset_bundle.is_loaded(state)
    }
}

impl Update<SpriteAssets> for InGameScreen {
    fn update(&mut self, state: &mut GameState<SpriteAssets>) {
        if self.asset_bundle.is_loaded(state) {
            let dt = state.delta_time;

            let screen = Vec2::new(
                state.window_size.width as f32,
                state.window_size.height as f32,
            );
            let half_screen = Vec2::new(screen.x / 2.0, screen.y / 2.0);
            let half_size = Vec2::new(200.0, 200.0);

            self.sprite1.x += self.velocity1.x * dt;
            self.sprite1.y += self.velocity1.y * dt;

            let sprite1_screen_x = self.sprite1.x + half_screen.x;
            let sprite1_screen_y = self.sprite1.y + half_screen.y;

            if sprite1_screen_x < half_size.x || sprite1_screen_x > screen.x - half_size.x {
                self.velocity1.x *= -1.0;
            }
            if sprite1_screen_y < half_size.y || sprite1_screen_y > screen.y - half_size.y {
                self.velocity1.y *= -1.0;
            }

            self.sprite2.x += self.velocity2.x * dt;
            self.sprite2.y += self.velocity2.y * dt;

            let sprite2_screen_x = self.sprite2.x + half_screen.x;
            let sprite2_screen_y = self.sprite2.y + half_screen.y;

            if sprite2_screen_x < half_size.x || sprite2_screen_x > screen.x - half_size.x {
                self.velocity2.x *= -1.0;
            }
            if sprite2_screen_y < half_size.y || sprite2_screen_y > screen.y - half_size.y {
                self.velocity2.y *= -1.0;
            }

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
