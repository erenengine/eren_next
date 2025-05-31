use eren_2d::{
    app::{App, AppConfig},
    game_world::{asset_bundle::AssetBundle, sprite::Sprite, state::GameState, update::Update},
};
use eren_core::render_world::common::gpu::GraphicsLibrary;
use rand::Rng;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
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
    sprites: Vec<Sprite<SpriteAssets>>,
    velocities: Vec<Vec2>,
}

impl InGameScreen {
    pub fn new() -> Self {
        let mut sprites = Vec::with_capacity(10_000);
        let mut velocities = Vec::with_capacity(10_000);
        let mut rng = rand::rng();

        let window_width = 1280.0;
        let window_height = 720.0;

        for _ in 0..100_000 {
            let x = rng.random_range(-window_width / 2.0..window_width / 2.0);
            let y = rng.random_range(-window_height / 2.0..window_height / 2.0);
            sprites.push(Sprite::new(x, y, SpriteAssets::TestSprite));

            let vx = rng.random_range(-2000.0..2000.0);
            let vy = rng.random_range(-2000.0..2000.0);
            velocities.push(Vec2::new(vx, vy));
        }

        Self {
            asset_bundle: AssetBundle::new(vec![(
                SpriteAssets::TestSprite,
                "examples/assets/test_sprite.png".into(),
            )]),
            sprites,
            velocities,
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
            let half_size = Vec2::new(32.0, 32.0);

            for (sprite, velocity) in self.sprites.iter_mut().zip(self.velocities.iter_mut()) {
                sprite.x += velocity.x * dt;
                sprite.y += velocity.y * dt;

                let sprite_screen_x = sprite.x + half_screen.x;
                let sprite_screen_y = sprite.y + half_screen.y;

                if sprite_screen_x < half_size.x || sprite_screen_x > screen.x - half_size.x {
                    velocity.x *= -1.0;
                }
                if sprite_screen_y < half_size.y || sprite_screen_y > screen.y - half_size.y {
                    velocity.y *= -1.0;
                }

                sprite.update(state);
            }
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
