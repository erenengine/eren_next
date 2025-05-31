use eren_core::render_world::wgpu::engine::WgpuEngine;

use crate::game_world::{state::GameState, update::Update};

pub struct WgpuEngine2D<R, SA> {
    root_node: R,
    game_state: GameState<SA>,
}

impl<R, SA> WgpuEngine2D<R, SA> {
    pub fn new(root_node: R) -> Self {
        Self {
            root_node,
            game_state: GameState::new(),
        }
    }
}

impl<R: Update<SA>, SA> WgpuEngine for WgpuEngine2D<R, SA> {
    fn on_gpu_resources_ready(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {}

    fn on_gpu_resources_lost(&mut self) {}

    fn update(
        &mut self,
        surface_texture_view: &wgpu::TextureView,
        encoder: &mut wgpu::CommandEncoder,
    ) {
        self.root_node.update(&mut self.game_state);
    }
}
