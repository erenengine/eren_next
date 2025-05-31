use std::hash::Hash;

use eren_core::render_world::wgpu::engine::WgpuEngine;

use crate::game_world::{state::GameState, update::Update};

use super::{
    asset_managers::sprite_asset_manager::WgpuSpriteAssetManager,
    renderers::sprite_renderer::{SpriteRenderCommand, WgpuSpriteRenderer},
};

pub struct WgpuEngine2D<R, SA> {
    root_node: R,
    game_state: GameState<SA>,

    sprite_asset_manager: WgpuSpriteAssetManager<SA>,
    sprite_renderer: WgpuSpriteRenderer,
}

impl<R, SA: Eq + Hash + Clone> WgpuEngine2D<R, SA> {
    pub fn new(root_node: R) -> Self {
        Self {
            root_node,
            game_state: GameState::new(),

            sprite_asset_manager: WgpuSpriteAssetManager::new(),
            sprite_renderer: WgpuSpriteRenderer::new(),
        }
    }
}

impl<R: Update<SA>, SA: Eq + Hash + Copy> WgpuEngine for WgpuEngine2D<R, SA> {
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
        command_encoder: &mut wgpu::CommandEncoder,
    ) {
        for (asset, path) in self.game_state.sprite_assets.pending.drain() {
            self.sprite_asset_manager.load_sprite(asset, path); // sync
            self.game_state.sprite_assets.ready.push(asset);
        }

        self.root_node.update(&mut self.game_state);

        let mut render_commands: Vec<SpriteRenderCommand> = vec![];
        for render_request in self.game_state.render_requests.drain(..) {
            let texture_view = self
                .sprite_asset_manager
                .get_texture_view(render_request.sprite_asset_id);
            if let Some(texture_view) = texture_view {
                render_commands.push(SpriteRenderCommand {
                    x: render_request.x,
                    y: render_request.y,
                    texture_view: texture_view.clone(),
                });
            }
        }

        self.sprite_renderer
            .render(surface_texture_view, command_encoder, render_commands);
    }
}
