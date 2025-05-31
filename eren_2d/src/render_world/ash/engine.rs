use eren_core::render_world::ash::engine::AshEngine;

use crate::game_world::{state::GameState, update::Update};

use winit::dpi::PhysicalSize;

pub struct AshEngine2D<R, SA> {
    root_node: R,
    game_state: GameState<SA>,
}

impl<R, SA> AshEngine2D<R, SA> {
    pub fn new(root_node: R) -> Self {
        Self {
            root_node,
            game_state: GameState::new(),
        }
    }
}

impl<R: Update<SA>, SA> AshEngine for AshEngine2D<R, SA> {
    fn on_gpu_resources_ready(&mut self) {}

    fn on_gpu_resources_lost(&mut self) {}

    fn on_window_resized(&mut self, window_size: PhysicalSize<u32>, window_scale_factor: f64) {}

    fn update(&mut self) {
        self.root_node.update(&mut self.game_state);
    }
}
