use std::{hash::Hash, time::Instant};

use eren_core::render_world::wgpu::engine::WgpuEngine;
use winit::dpi::PhysicalSize;

use crate::game_world::{state::GameState, transform::GlobalTransform, game_node::GameNode};

use super::{
    asset_managers::sprite_asset_manager::WgpuSpriteAssetManager,
    renderers::sprite_renderer::{SpriteRenderCommand, WgpuSpriteRenderer},
};

pub struct WgpuEngine2D<R, SA>
where
    SA: Eq + Hash + Copy + Ord, // WgpuSpriteRenderer에서 Ord를 요구하므로 추가
{
    game_state: GameState<SA>,
    root_node: R,
    default_global_transform: GlobalTransform,

    sprite_asset_manager: WgpuSpriteAssetManager<SA>,
    sprite_renderer: WgpuSpriteRenderer<SA>, // SA 제네릭 명시

    last_frame_time: Instant,
}

impl<R, SA> WgpuEngine2D<R, SA>
where
    R: GameNode<SA>, // R의 제약은 그대로
    SA: Eq + Hash + Copy + Ord + Clone, // WgpuSpriteRenderer에서 Ord 요구, SA가 Clone 되어야 할 수 있음 (GameState 등에서)
                                        // SpriteRenderCommand<SA>의 sprite_asset_id가 SA를 값으로 가지므로 Clone 필요
{
    pub fn new(root_node: R) -> Self {
        Self {
            game_state: GameState::new(),
            root_node,
            default_global_transform: GlobalTransform::new(),

            sprite_asset_manager: WgpuSpriteAssetManager::new(),
            sprite_renderer: WgpuSpriteRenderer::new(), // new()는 SA 제약을 따름

            last_frame_time: Instant::now(),
        }
    }
}

impl<R, SA> WgpuEngine for WgpuEngine2D<R, SA>
where
    R: GameNode<SA>,
    SA: Eq + Hash + Copy + Ord + Clone, // 여기도 일관성 있게 제약 조건 업데이트
{
    fn on_gpu_resources_ready(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        window_size: PhysicalSize<u32>,
        window_scale_factor: f64,
    ) {
        self.sprite_asset_manager
            .on_gpu_resources_ready(device, queue);
        self.sprite_renderer // SA 타입에 대한 on_gpu_resources_ready 호출
            .on_gpu_resources_ready(
                device,
                queue,
                surface_format,
                window_size,
                window_scale_factor,
            );
    }

    fn on_gpu_resources_lost(&mut self) {
        self.sprite_asset_manager.on_gpu_resources_lost();
        self.sprite_renderer.on_gpu_resources_lost(); // SA 타입에 대한 호출
    }

    fn on_window_resized(&mut self, window_size: PhysicalSize<u32>, window_scale_factor: f64) {
        self.game_state.window_size = window_size;
        self.sprite_renderer
            .on_window_resized(window_size, window_scale_factor); // SA 타입에 대한 호출
    }

    fn update(
        &mut self,
        surface_texture_view: &wgpu::TextureView,
        command_encoder: &mut wgpu::CommandEncoder,
    ) {
        let now = Instant::now();
        self.game_state.delta_time = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;

        println!("FPS: {}", 1.0 / self.game_state.delta_time);

        for (asset, path) in self.game_state.sprite_assets.pending.drain() {
            self.sprite_asset_manager.load_sprite(asset, path);
            self.game_state.sprite_assets.ready.push(asset);
        }

        self.root_node
            .update(&mut self.game_state, &self.default_global_transform);

        let mut render_commands: Vec<SpriteRenderCommand<SA>> = vec![];
        for render_request in self.game_state.render_requests.drain(..) {
            // sprite_asset_id는 이미 SA 타입이므로 그대로 사용
            let asset_id = render_request.sprite_asset_id;
            let texture = self.sprite_asset_manager.get_texture(asset_id); // SA는 Copy이므로 문제 없음

            if let Some(texture_ref) = texture {
                // get_texture가 &WgpuTexture를 반환한다고 가정
                render_commands.push(SpriteRenderCommand {
                    x: render_request.x,
                    y: render_request.y,
                    sprite_asset_id: asset_id,
                    texture: texture_ref.clone(), // WgpuTexture의 clone()은 여전히 호출
                                                  // WgpuTexture가 Arc 등으로 잘 감싸져 있다면 이 clone은 가벼움
                });
            }
        }
        // render 함수는 이제 Vec<SpriteRenderCommand<SA>>를 받음
        self.sprite_renderer
            .render(surface_texture_view, command_encoder, render_commands);
    }
}
