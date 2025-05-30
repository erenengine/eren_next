use crate::game_world::state::GameState;

use super::{
    asset_loaders::sprite_loader::WgpuSpriteLoader,
    render_passes::sprite_render_pass::WgpuSpriteRenderPass,
};

pub struct WgpuRenderer<AssetId> {
    sprite_loader: WgpuSpriteLoader,
    sprite_render_pass: WgpuSpriteRenderPass<AssetId>,
}

impl<AssetId> WgpuRenderer<AssetId> {
    pub fn new() -> Self {
        Self {
            sprite_loader: WgpuSpriteLoader::new(),
            sprite_render_pass: WgpuSpriteRenderPass::new(),
        }
    }

    pub fn render(&mut self, state: &GameState<AssetId>) {}
}
