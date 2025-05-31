use std::{hash::Hash, time::Instant};

use eren_core::render_world::wgpu::engine::WgpuEngine;
use winit::dpi::PhysicalSize;

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

    last_frame_time: Instant,
}

impl<R, SA: Eq + Hash + Clone> WgpuEngine2D<R, SA> {
    pub fn new(root_node: R) -> Self {
        Self {
            root_node,
            game_state: GameState::new(),

            sprite_asset_manager: WgpuSpriteAssetManager::new(),
            sprite_renderer: WgpuSpriteRenderer::new(),

            last_frame_time: Instant::now(),
        }
    }
}

impl<R: Update<SA>, SA: Eq + Hash + Copy> WgpuEngine for WgpuEngine2D<R, SA> {
    fn on_gpu_resources_ready(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        window_size: PhysicalSize<u32>,
    ) {
        self.sprite_asset_manager
            .on_gpu_resources_ready(device, queue);
        self.sprite_renderer
            .on_gpu_resources_ready(device, queue, window_size);
    }

    fn on_gpu_resources_lost(&mut self) {
        self.sprite_asset_manager.on_gpu_resources_lost();
        self.sprite_renderer.on_gpu_resources_lost();
    }

    fn on_window_resized(&mut self, window_size: PhysicalSize<u32>) {
        self.game_state.window_size = window_size;
        self.sprite_renderer.on_window_resized(window_size);
    }

    fn update(
        &mut self,
        surface_texture_view: &wgpu::TextureView,
        command_encoder: &mut wgpu::CommandEncoder,
    ) {
        let now = Instant::now();
        self.game_state.delta_time = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;

        for (asset, path) in self.game_state.sprite_assets.pending.drain() {
            self.sprite_asset_manager.load_sprite(asset, path); // sync
            self.game_state.sprite_assets.ready.push(asset);
        }

        self.root_node.update(&mut self.game_state);

        let mut render_commands: Vec<SpriteRenderCommand> = vec![];
        for render_request in self.game_state.render_requests.drain(..) {
            let texture = self
                .sprite_asset_manager
                .get_texture(render_request.sprite_asset_id);
            if let Some(texture) = texture {
                render_commands.push(SpriteRenderCommand {
                    x: render_request.x,
                    y: render_request.y,
                    texture: texture.clone(),
                });
            }
        }

        self.sprite_renderer
            .render(surface_texture_view, command_encoder, render_commands);
    }
}
