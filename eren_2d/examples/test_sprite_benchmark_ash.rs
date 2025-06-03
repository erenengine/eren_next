use eren_2d::{
    app::{App, AppConfig},
    game_world::{
        asset_bundle::AssetBundle, nodes::game_node::GameNode, nodes::sprite_node::SpriteNode,
        state::GameState, transform::GlobalTransform,
    },
};
use eren_core::render_world::common::gpu::GraphicsLibrary;
use glam::Vec2;
use rand::Rng;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum SpriteAssets {
    Logo,
    TestSprite,
}

struct RootNode {
    loading_screen: Option<LoadingScreen>,
    in_game_screen: InGameScreen,
}

impl RootNode {
    pub fn new() -> Self {
        Self {
            loading_screen: Some(LoadingScreen::new()),
            in_game_screen: InGameScreen::new(),
        }
    }
}

impl GameNode<SpriteAssets> for RootNode {
    fn update(
        &mut self,
        game_state: &mut GameState<SpriteAssets>,
        parent_global_transform: &GlobalTransform,
    ) {
        if self.in_game_screen.is_asset_loaded(game_state) {
            self.loading_screen = None;
        }

        if let Some(loading_screen) = self.loading_screen.as_mut() {
            loading_screen.update(game_state, parent_global_transform);
        } else {
            self.in_game_screen
                .update(game_state, parent_global_transform);
        }
    }
}

struct LoadingScreen {
    asset_bundle: AssetBundle<SpriteAssets>,
    logo: SpriteNode<SpriteAssets>,
}

impl LoadingScreen {
    pub fn new() -> Self {
        Self {
            asset_bundle: AssetBundle::new(vec![(
                SpriteAssets::Logo,
                "examples/assets/logo.png".into(),
            )]),
            logo: SpriteNode::new(SpriteAssets::Logo),
        }
    }
}

impl GameNode<SpriteAssets> for LoadingScreen {
    fn update(
        &mut self,
        game_state: &mut GameState<SpriteAssets>,
        parent_global_transform: &GlobalTransform,
    ) {
        if self.asset_bundle.is_loaded(game_state) {
            self.logo.update(game_state, parent_global_transform);
        }
    }
}

struct InGameScreen {
    asset_bundle: AssetBundle<SpriteAssets>,
    sprites: Vec<SpriteNode<SpriteAssets>>,
    velocities: Vec<Vec2>,
}

impl InGameScreen {
    pub fn new() -> Self {
        let mut sprites: Vec<SpriteNode<SpriteAssets>> = Vec::with_capacity(100_000);
        let mut velocities = Vec::with_capacity(100_000);
        let mut rng = rand::rng();

        let window_width = 1280.0;
        let window_height = 720.0;

        for _ in 0..100_000 {
            let mut sprite = SpriteNode::new(SpriteAssets::TestSprite);

            let x = rng.random_range(-window_width / 2.0..window_width / 2.0);
            let y = rng.random_range(-window_height / 2.0..window_height / 2.0);
            sprite.transform.set_position(Vec2::new(x, y));

            let scale = rng.random_range(0.5..2.0);
            sprite.transform.set_scale(Vec2::splat(scale));

            let rotation = rng.random_range(0.0..2.0 * std::f32::consts::PI);
            sprite.transform.set_rotation(rotation);

            let alpha = rng.random_range(0.0..1.0);
            sprite.transform.set_alpha(alpha);

            sprites.push(sprite);

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

    pub fn is_asset_loaded(&mut self, game_state: &mut GameState<SpriteAssets>) -> bool {
        self.asset_bundle.is_loaded(game_state)
    }
}

impl GameNode<SpriteAssets> for InGameScreen {
    fn update(
        &mut self,
        game_state: &mut GameState<SpriteAssets>,
        parent_global_transform: &GlobalTransform,
    ) {
        if self.asset_bundle.is_loaded(game_state) {
            let dt = game_state.delta_time;

            let screen = Vec2::new(
                game_state.window_size.width as f32,
                game_state.window_size.height as f32,
            );
            let half_screen = Vec2::new(screen.x / 2.0, screen.y / 2.0);
            let half_size = Vec2::new(32.0, 32.0);

            for (sprite, velocity) in self.sprites.iter_mut().zip(self.velocities.iter_mut()) {
                sprite.transform.set_position(Vec2::new(
                    sprite.transform.position().x + velocity.x * dt,
                    sprite.transform.position().y + velocity.y * dt,
                ));

                let sprite_screen_x = sprite.transform.position().x + half_screen.x;
                let sprite_screen_y = sprite.transform.position().y + half_screen.y;

                if sprite_screen_x < half_size.x || sprite_screen_x > screen.x - half_size.x {
                    velocity.x *= -1.0;
                }
                if sprite_screen_y < half_size.y || sprite_screen_y > screen.y - half_size.y {
                    velocity.y *= -1.0;
                }

                sprite.update(game_state, parent_global_transform);
            }
        }
    }
}

fn main() {
    App::new(
        AppConfig {
            window_width: 1280,
            window_height: 720,
            window_title: "Test Sprite Benchmark (Ash)".to_string(),
            graphics_library: GraphicsLibrary::Ash,
        },
        RootNode::new(),
    )
    .run();
}
