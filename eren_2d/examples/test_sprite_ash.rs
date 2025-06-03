use eren_2d::{
    app::{App, AppConfig},
    game_world::{
        asset_bundle::AssetBundle, nodes::game_node::GameNode, nodes::sprite_node::SpriteNode,
        state::GameState, transform::GlobalTransform,
    },
};
use eren_core::render_world::common::gpu::GraphicsLibrary;

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
    sprite: SpriteNode<SpriteAssets>,
}

impl InGameScreen {
    pub fn new() -> Self {
        Self {
            asset_bundle: AssetBundle::new(vec![(
                SpriteAssets::TestSprite,
                "examples/assets/test_sprite.png".into(),
            )]),
            sprite: SpriteNode::new(SpriteAssets::TestSprite),
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
            self.sprite.update(game_state, parent_global_transform);
        }
    }
}

fn main() {
    App::new(
        AppConfig {
            window_width: 1280,
            window_height: 720,
            window_title: "Test Sprite (Ash)".to_string(),
            graphics_library: GraphicsLibrary::Ash,
        },
        RootNode::new(),
    )
    .run();
}
