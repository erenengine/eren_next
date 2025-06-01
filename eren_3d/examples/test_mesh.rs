use eren_3d::{
    app::{App, AppConfig},
    game_world::{
        asset_bundle::AssetBundle,
        nodes::{game_node::GameNode, model_node::ModelNode},
        state::GameState,
        transform::GlobalTransform,
    },
};
use eren_core::render_world::common::gpu::GraphicsLibrary;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum ModelAssets {
    Character,
}

struct InGameScreen {
    asset_bundle: AssetBundle<ModelAssets>,
    model: ModelNode<ModelAssets>,
}

impl InGameScreen {
    pub fn new() -> Self {
        Self {
            asset_bundle: AssetBundle::new(vec![(
                ModelAssets::Character,
                "examples/assets/kenney-mini-characters/character-female-a.glb".into(),
            )]),
            model: ModelNode::new(ModelAssets::Character),
        }
    }
}

impl GameNode<ModelAssets> for InGameScreen {
    fn update(
        &mut self,
        game_state: &mut GameState<ModelAssets>,
        parent_global_transform: &GlobalTransform,
    ) {
        if self.asset_bundle.is_loaded(game_state) {
            self.model.update(game_state, parent_global_transform);
        }
    }
}

fn main() {
    App::new(
        AppConfig {
            window_width: 1280,
            window_height: 720,
            window_title: "Test Mesh".to_string(),
            graphics_library: GraphicsLibrary::Wgpu,
        },
        InGameScreen::new(),
    )
    .run();
}
