use ash::vk;
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
    fn on_gpu_resources_ready(
        &mut self,
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: ash::Device,
        graphics_queue: vk::Queue,
        command_pool: vk::CommandPool,
        swapchain_format: vk::Format,
        window_size: PhysicalSize<u32>,
        scale_factor: f64,
        max_sprites: u32,
    ) {
    }

    fn on_gpu_resources_lost(&mut self) {}

    fn on_window_resized(&mut self, window_size: PhysicalSize<u32>, scale_factor: f64) {}

    fn update(
        &mut self,
        cb: vk::CommandBuffer,
        framebuffer: vk::Framebuffer,
        render_area: vk::Rect2D,
        viewport: vk::Viewport,
        scissor: vk::Rect2D,
    ) {
    }
}
