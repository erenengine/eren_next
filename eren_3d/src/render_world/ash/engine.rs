use eren_core::render_world::ash::engine::AshEngine;

use crate::game_world::{nodes::game_node::GameNode, state::GameState, transform::GlobalTransform};

use winit::dpi::PhysicalSize;

pub struct AshEngine3D<R, SA> {
    game_state: GameState<SA>,
    root_node: R,
    default_global_transform: GlobalTransform,
}

impl<R, SA> AshEngine3D<R, SA> {
    pub fn new(root_node: R) -> Self {
        Self {
            game_state: GameState::new(),
            root_node,
            default_global_transform: GlobalTransform::new(),
        }
    }
}

impl<R: GameNode<SA>, SA> AshEngine for AshEngine3D<R, SA> {
    fn on_gpu_resources_ready(&mut self) {}

    fn on_gpu_resources_lost(&mut self) {}

    fn on_window_resized(&mut self, window_size: PhysicalSize<u32>, window_scale_factor: f64) {}

    fn update(&mut self) {
        self.root_node
            .update(&mut self.game_state, &self.default_global_transform);
    }
}
