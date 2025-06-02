use eren_core::render_world::ash::engine::AshEngine;

use crate::game_world::{nodes::game_node::GameNode, state::GameState, transform::GlobalTransform};

use ash::{Device, Instance, vk};
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
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        device: Device, // Pass by value as it's cloned
        graphics_queue: vk::Queue,
        command_pool: vk::CommandPool,
        swapchain_format: vk::Format,
        render_pass: vk::RenderPass,
        // Framebuffers are implicitly known via image_index and what renderer holds
        window_size: PhysicalSize<u32>,
        scale_factor: f64,
        max_sprites: u32,
        frames_in_flight: usize, // Engine needs to know this for its own per-frame resources
    ) {
    }

    fn on_gpu_resources_lost(&mut self) {}

    fn on_window_resized(
        &mut self,
        new_size: PhysicalSize<u32>,
        new_scale_factor: f64,
        // Engine might need new renderpass/framebuffers if format changes,
        // but for typical resize, just size/scale is enough for its internal logic.
        // The GpuResourceManager handles swapchain recreation.
    ) {
    }

    fn update(
        &mut self,
        command_buffer: vk::CommandBuffer, // Command buffer for the current frame
        image_index: u32,                  // Index of the current swapchain image
        current_frame_index: usize, // Index for per-frame resources (0..MAX_FRAMES_IN_FLIGHT-1)
                                    // Pass other necessary items like current framebuffer if engine doesn't manage them directly
                                    // For this setup, engine will get framebuffer from its internal list using image_index
                                    // and render_area/viewport from its internal state or window_size.
    ) {
    }
}
