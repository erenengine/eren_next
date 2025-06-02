use ash::vk;
use eren_core::render_world::ash::engine::AshEngine;
use std::{hash::Hash, time::Instant};
use winit::dpi::PhysicalSize;

use crate::game_world::{
    nodes::game_node::GameNode,
    state::GameState, // Alias to avoid conflict
    transform::GlobalTransform,
};

use super::{
    asset_managers::sprite_asset_manager::AshSpriteAssetManager,
    renderers::sprite_renderer::{AshSpriteRenderer, SpriteRenderCommand},
};

pub struct AshEngine2D<R, SA>
where
    SA: Eq + Hash + Clone + Copy,
{
    game_state: GameState<SA>,
    root_node: R,
    default_global_transform: GlobalTransform,

    sprite_asset_manager: AshSpriteAssetManager<SA>,
    sprite_renderer: AshSpriteRenderer<SA>,

    last_frame_time: Instant,

    device: Option<ash::Device>,
    // Store these to pass to renderer
    render_pass: Option<vk::RenderPass>,
    // Framebuffers for each swapchain image, managed by GpuResourceManager, renderer needs one per frame
    swapchain_framebuffers: Vec<vk::Framebuffer>,
    window_size: PhysicalSize<u32>,
    scale_factor: f64,
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
            device: None,
            render_pass: None,
            swapchain_framebuffers: Vec::new(),
            window_size: PhysicalSize::new(0, 0), // Initialized in on_gpu_resources_ready
            scale_factor: 1.0,
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
        device: ash::Device, // Takes ownership/Arc
        graphics_queue: vk::Queue,
        command_pool: vk::CommandPool,
        swapchain_format: vk::Format, // Used by renderer for pipeline
        render_pass: vk::RenderPass,  // Used by renderer for pipeline
        // swapchain_images: &Vec<vk::Image>, // Not directly used by engine, but by GpuRM
        // swapchain_image_views: &Vec<vk::ImageView>, // For creating framebuffers
        // swapchain_framebuffers_param: &Vec<vk::Framebuffer>, // Use this to store
        window_size: PhysicalSize<u32>,
        scale_factor: f64,
        max_sprites: u32,
        _frames_in_flight: usize, // For asset manager/renderer if they have per-frame resources
    ) {
        self.window_size = window_size;
        self.scale_factor = scale_factor;
        self.render_pass = Some(render_pass);
        // self.swapchain_framebuffers = swapchain_framebuffers_param.clone(); // Store framebuffers

        // It's important that device is cloned if multiple components need it,
        // or an Arc<Device> is used. Here, we clone for each manager.
        let device_clone_for_assets = device.clone();
        let device_clone_for_renderer = device.clone();
        let phys_mem_props =
            unsafe { instance.get_physical_device_memory_properties(physical_device) };

        self.sprite_asset_manager.on_gpu_resources_ready(
            device_clone_for_assets, // Pass cloned device
            phys_mem_props.clone(),  // Clone phys_mem_props
            graphics_queue,
            command_pool, // Asset manager might need its own pool if uploads are frequent/parallel
            max_sprites,
        );

        let sprite_texture_set_layout = self
            .sprite_asset_manager
            .descriptor_set_layout()
            .expect("Sprite asset manager descriptor set layout not initialized");

        self.sprite_renderer.on_gpu_resources_ready(
            instance,
            physical_device,
            device_clone_for_renderer, // Pass cloned device
            phys_mem_props,            // Use original or cloned
            render_pass,
            sprite_texture_set_layout,
            window_size,
            scale_factor,
            // max_sprites, // Renderer uses this for initial instance buffer capacity
        );
        // Update game state with initial window size
        self.game_state.window_size = window_size;
    }

    fn on_gpu_resources_lost(&mut self) {
        self.sprite_asset_manager.on_gpu_resources_lost();
        self.sprite_renderer.on_gpu_resources_lost();
        self.render_pass = None;
        self.swapchain_framebuffers.clear();
    }

    fn on_window_resized(&mut self, new_size: PhysicalSize<u32>, new_scale_factor: f64) {
        self.window_size = new_size;
        self.scale_factor = new_scale_factor;
        self.game_state.window_size = new_size; // Update game state
        // Renderer needs to update its UBO
        self.sprite_renderer
            .on_window_resized(new_size, new_scale_factor);
        // GpuResourceManager handles swapchain recreation. Engine receives new framebuffers/renderpass if they change.
        // For this simplified model, we assume renderpass structure remains same, only extent changes.
    }

    fn update(
        &mut self,
        command_buffer: vk::CommandBuffer,
        image_index: u32,            // Index of the current swapchain image
        _current_frame_index: usize, // For per-frame resources (0..MAX_FRAMES_IN_FLIGHT-1)
    ) {
        let now = Instant::now();
        self.game_state.delta_time = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;

        // Process pending asset loads
        // Create a temporary vec to collect assets to avoid borrowing issues with self.game_state
        let pending_to_load: Vec<(SA, String)> =
            self.game_state.sprite_assets.pending.drain().collect();
        for (asset_id, path) in pending_to_load {
            self.sprite_asset_manager
                .load_sprite(asset_id.clone(), path);
            self.game_state.sprite_assets.ready.push(asset_id); // Mark as ready for game logic
        }

        // Update game logic (nodes)
        self.root_node
            .update(&mut self.game_state, &self.default_global_transform);

        // Collect render commands
        let mut sprite_render_commands: Vec<SpriteRenderCommand<SA>> =
            Vec::with_capacity(self.game_state.render_requests.len());

        for req in self.game_state.render_requests.drain(..) {
            if let Some(gpu_res) = self
                .sprite_asset_manager
                .get_gpu_resource(&req.sprite_asset_id)
            {
                sprite_render_commands.push(SpriteRenderCommand {
                    size: gpu_res.size,
                    matrix: req.matrix,
                    alpha: req.alpha,
                    sprite_asset_id: req.sprite_asset_id.clone(), // Clone asset ID
                    descriptor_set: gpu_res.descriptor_set,
                });
            } else {
                // Log warning: GPU resource for sprite not found (still loading or error)
                // eprintln!("Warning: GPU resource for sprite {:?} not found.", req.sprite_asset_id);
            }
        }

        // --- Perform Rendering ---
        // AshGpuResourceManager should have started the render pass if it's common.
        // Or, this engine could start its own specific render pass here.
        // For this example, assume AshGpuResourceManager began a render pass targeting
        // the swapchain_framebuffers[image_index as usize].

        // Renderer needs viewport and scissor for the current swapchain extent
        let viewport = vk::Viewport {
            x: 0.0,
            y: 0.0,
            width: self.window_size.width as f32,
            height: self.window_size.height as f32,
            min_depth: 0.0,
            max_depth: 1.0,
        };
        let scissor = vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: vk::Extent2D {
                width: self.window_size.width,
                height: self.window_size.height,
            },
        };

        let render_pass_begin_info = vk::RenderPassBeginInfo::default()
            .render_pass(self.render_pass.unwrap())
            .framebuffer(self.swapchain_framebuffers[image_index as usize])
            .render_area(scissor) // Scissor usually same as render area for full screen
            .clear_values(&[vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.1, 0.1, 0.1, 1.0],
                },
            }]);

        unsafe {
            self.device.as_ref().unwrap().cmd_begin_render_pass(
                command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
        }

        self.sprite_renderer.render(
            command_buffer,
            // framebuffer, // Implicit from render pass started by GpuRM
            // render_area, // Implicit
            viewport,
            scissor,
            &sprite_render_commands,
        );

        unsafe {
            self.device
                .as_ref()
                .unwrap()
                .cmd_end_render_pass(command_buffer);
        }
        // AshGpuResourceManager will end the command buffer and submit it.
    }
}
