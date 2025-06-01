use std::{hash::Hash, time::Instant};

use eren_core::render_world::wgpu::engine::WgpuEngine;
use winit::dpi::PhysicalSize;

use crate::game_world::{game_node::GameNode, state::GameState, transform::GlobalTransform};

use super::{
    asset_managers::sprite_asset_manager::WgpuSpriteAssetManager,
    bind_group_layout::create_sprite_bind_group_layout::create_sprite_bind_group_layout,
    renderers::sprite_renderer::{SpriteRenderCommand, WgpuSpriteRenderer},
};

pub struct WgpuEngine2D<R, SA> {
    game_state: GameState<SA>,
    root_node: R,
    default_global_transform: GlobalTransform,

    sprite_asset_manager: WgpuSpriteAssetManager<SA>,
    sprite_renderer: WgpuSpriteRenderer<SA>,

    last_frame_time: Instant,
}

impl<R, SA> WgpuEngine2D<R, SA>
where
    R: GameNode<SA>,
    SA: Eq + Hash + Copy,
{
    pub fn new(root_node: R) -> Self {
        Self {
            game_state: GameState::new(),
            root_node,
            default_global_transform: GlobalTransform::new(),

            sprite_asset_manager: WgpuSpriteAssetManager::new(),
            sprite_renderer: WgpuSpriteRenderer::new(),

            last_frame_time: Instant::now(),
        }
    }
}

impl<R, SA> WgpuEngine for WgpuEngine2D<R, SA>
where
    R: GameNode<SA>,
    SA: Eq + Hash + Copy,
{
    fn on_gpu_resources_ready(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        window_size: PhysicalSize<u32>,
        window_scale_factor: f64,
    ) {
        let sprite_bind_group_layout = create_sprite_bind_group_layout(device);
        self.sprite_asset_manager
            .on_gpu_resources_ready(device, queue, &sprite_bind_group_layout);
        self.sprite_renderer.on_gpu_resources_ready(
            device,
            queue,
            surface_format,
            &sprite_bind_group_layout,
            window_size,
            window_scale_factor,
        );
    }

    fn on_gpu_resources_lost(&mut self) {
        self.sprite_asset_manager.on_gpu_resources_lost();
        self.sprite_renderer.on_gpu_resources_lost();
    }

    fn on_window_resized(&mut self, window_size: PhysicalSize<u32>, window_scale_factor: f64) {
        self.game_state.window_size = window_size;
        self.sprite_renderer
            .on_window_resized(window_size, window_scale_factor);
    }

    fn update(
        &mut self,
        surface_texture_view: &wgpu::TextureView,
        command_encoder: &mut wgpu::CommandEncoder,
    ) {
        let now = Instant::now();
        self.game_state.delta_time = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;

        // TODO: 제거
        println!("FPS: {}", 1.0 / self.game_state.delta_time);

        for (asset, path) in self.game_state.sprite_assets.pending.drain() {
            self.sprite_asset_manager.load_sprite(asset, path);
            self.game_state.sprite_assets.ready.push(asset);
        }

        self.root_node
            .update(&mut self.game_state, &self.default_global_transform);

        let mut render_commands: Vec<SpriteRenderCommand<SA>> = vec![];
        for render_request in self.game_state.render_requests.drain(..) {
            let asset_id = render_request.sprite_asset_id;
            let gpu_resource = self.sprite_asset_manager.get_gpu_resource(asset_id);

            if let Some(gpu_resource_ref) = gpu_resource {
                render_commands.push(SpriteRenderCommand {
                    position: render_request.position,
                    size: gpu_resource_ref.size,
                    scale: render_request.scale,
                    rotation: render_request.rotation,
                    alpha: render_request.alpha,
                    sprite_asset_id: asset_id,
                    bind_group: gpu_resource_ref.bind_group.clone(),
                });
            }
        }

        self.sprite_renderer
            .render(surface_texture_view, command_encoder, render_commands);
    }
}
