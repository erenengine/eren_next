use eren_core::render_world::wgpu::engine::WgpuEngine;

use crate::game_world::{state::GameState, update::Update};

use super::{
    asset_managers::sprite_asset_manager::WgpuSpriteAssetManager,
    renderers::sprite_renderer::{SpriteRenderCommand, WgpuSpriteRenderer},
};

pub struct WgpuEngine2D<R, SA> {
    root_node: R,
    game_state: GameState<SA>,

    sprite_asset_manager: WgpuSpriteAssetManager,
    sprite_renderer: WgpuSpriteRenderer,
}

impl<R, SA> WgpuEngine2D<R, SA> {
    pub fn new(root_node: R) -> Self {
        Self {
            root_node,
            game_state: GameState::new(),

            sprite_asset_manager: WgpuSpriteAssetManager::new(),
            sprite_renderer: WgpuSpriteRenderer::new(),
        }
    }
}

impl<R: Update<SA>, SA> WgpuEngine for WgpuEngine2D<R, SA> {
    fn on_gpu_resources_ready(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.sprite_asset_manager
            .on_gpu_resources_ready(device, queue);
        self.sprite_renderer.on_gpu_resources_ready(device, queue);
    }

    fn on_gpu_resources_lost(&mut self) {
        self.sprite_asset_manager.on_gpu_resources_lost();
        self.sprite_renderer.on_gpu_resources_lost();
    }

    fn update(
        &mut self,
        surface_texture_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        // TODO: Ensure assets are loaded

        self.root_node.update(&mut self.game_state);

        let render_commands: Vec<SpriteRenderCommand> = vec![];
        // TODO: Generate render commands

        self.sprite_renderer.render(render_commands);

        self.game_state.render_requests.clear();
    }
}
