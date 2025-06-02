use std::{hash::Hash, time::Instant};

use ash::vk;
use eren_core::render_world::ash::engine::AshEngine;
use winit::dpi::PhysicalSize;

use crate::game_world::{nodes::game_node::GameNode, state::GameState, transform::GlobalTransform};

use super::{
    asset_managers::sprite_asset_manager::AshSpriteAssetManager,
    renderers::sprite_renderer::AshSpriteRenderer,
};

pub struct AshEngine2D<R, SA> {
    game_state: GameState<SA>,
    root_node: R,
    default_global_transform: GlobalTransform,

    sprite_asset_manager: AshSpriteAssetManager<SA>,
    sprite_renderer: AshSpriteRenderer<SA>,

    last_frame_time: Instant,
}

impl<R, SA> AshEngine2D<R, SA>
where
    R: GameNode<SA>,
    SA: Eq + Hash + Copy + Clone,
{
    pub fn new(root_node: R) -> Self {
        Self {
            game_state: GameState::new(),
            root_node,
            default_global_transform: GlobalTransform::new(),
            sprite_asset_manager: AshSpriteAssetManager::new(),
            sprite_renderer: AshSpriteRenderer::new(),
            last_frame_time: Instant::now(),
        }
    }
}

impl<R, SA> AshEngine for AshEngine2D<R, SA>
where
    R: GameNode<SA>,
    SA: Eq + Hash + Copy + Clone,
{
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
        self.sprite_asset_manager.on_gpu_resources_ready(
            device.clone(),
            unsafe { instance.get_physical_device_memory_properties(physical_device) },
            graphics_queue,
            command_pool,
            max_sprites,
        );

        let sprite_set_layout = self
            .sprite_asset_manager
            .descriptor_set_layout()
            .expect("Sprite asset manager not initialised");

        self.sprite_renderer.on_gpu_resources_ready(
            instance,
            physical_device,
            device,
            swapchain_format,
            sprite_set_layout,
            window_size,
            scale_factor,
        );
    }

    fn on_gpu_resources_lost(&mut self) {
        self.sprite_asset_manager.on_gpu_resources_lost();
        self.sprite_renderer.on_gpu_resources_lost();
    }

    fn on_window_resized(&mut self, window_size: PhysicalSize<u32>, scale_factor: f64) {
        self.game_state.window_size = window_size;
        self.sprite_renderer
            .on_window_resized(window_size, scale_factor);
    }

    fn update(
        &mut self,
        command_buffer: vk::CommandBuffer,
        frame_buffer: vk::Framebuffer,
        render_area: vk::Rect2D,
        viewport: vk::Viewport,
        scissor: vk::Rect2D,
    ) {
        let now = Instant::now();
        self.game_state.delta_time = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;

        for (asset, path) in self.game_state.sprite_assets.pending.drain() {
            self.sprite_asset_manager.load_sprite(asset.clone(), path);
            self.game_state.sprite_assets.ready.push(asset);
        }

        self.root_node
            .update(&mut self.game_state, &self.default_global_transform);

        let mut sprite_commands: Vec<super::renderers::sprite_renderer::SpriteRenderCommand<SA>> =
            Vec::with_capacity(self.game_state.render_requests.len());

        for req in self.game_state.render_requests.drain(..) {
            if let Some(res) = self
                .sprite_asset_manager
                .get_gpu_resource(&req.sprite_asset_id)
            {
                sprite_commands.push(super::renderers::sprite_renderer::SpriteRenderCommand {
                    size: res.size,
                    matrix: req.matrix,
                    alpha: req.alpha,
                    sprite_asset_id: req.sprite_asset_id,
                    descriptor_set: res.descriptor_set,
                });
            }
        }

        self.sprite_renderer.render(
            command_buffer,
            frame_buffer,
            render_area,
            viewport,
            scissor,
            &sprite_commands,
        );
    }
}
