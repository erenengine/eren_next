use ash::{Device, vk};
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

    device: Option<Device>,
    render_pass: Option<vk::RenderPass>,
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
            window_size: PhysicalSize::new(0, 0),
            scale_factor: 1.0,
        }
    }
}

impl<R, SA> AshEngine for AshEngine2D<R, SA>
where
    R: GameNode<SA>,
    SA: Eq + Hash + Copy + Clone,
{
    #[allow(clippy::too_many_arguments)]
    fn on_gpu_resources_ready(
        &mut self,
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: ash::Device,
        graphics_queue: vk::Queue,
        command_pool: vk::CommandPool,
        swapchain_format: vk::Format,
        render_pass: vk::RenderPass,
        swapchain_framebuffers: Vec<vk::Framebuffer>,
        window_size: PhysicalSize<u32>,
        scale_factor: f64,
        max_sprites: u32,
        _frames_in_flight: usize,
    ) {
        self.window_size = window_size;
        self.scale_factor = scale_factor;

        self.device = Some(device.clone());
        self.render_pass = Some(render_pass);
        self.swapchain_framebuffers = swapchain_framebuffers;

        // AssetManager 초기화
        let device_clone_for_assets = device.clone();
        let device_clone_for_renderer = device.clone();
        let phys_mem_props =
            unsafe { instance.get_physical_device_memory_properties(physical_device) };

        self.sprite_asset_manager.on_gpu_resources_ready(
            device_clone_for_assets,
            phys_mem_props.clone(),
            graphics_queue,
            command_pool,
            max_sprites,
        );

        let sprite_texture_set_layout = self
            .sprite_asset_manager
            .descriptor_set_layout()
            .expect("Sprite asset manager descriptor set layout not initialized");

        // Renderer 초기화 (렌더 패스는 엔진에서만 다루므로 파라미터에서 제외)
        self.sprite_renderer.on_gpu_resources_ready(
            instance,
            physical_device,
            device_clone_for_renderer,
            phys_mem_props,
            sprite_texture_set_layout,
            window_size,
            scale_factor,
            max_sprites as usize,
        );

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
        self.game_state.window_size = new_size;
        self.sprite_renderer
            .on_window_resized(new_size, new_scale_factor);
    }

    /// ▶ 매 프레임 호출되는 업데이트 로직
    fn update(
        &mut self,
        command_buffer: vk::CommandBuffer,
        image_index: u32,
        _current_frame_index: usize,
    ) {
        // 1) 델타 타임 갱신
        let now = Instant::now();
        self.game_state.delta_time = now.duration_since(self.last_frame_time).as_secs_f32();
        self.last_frame_time = now;

        // 2) AssetManager에 로딩 요청된 스프라이트 처리
        let pending_to_load: Vec<(SA, String)> =
            self.game_state.sprite_assets.pending.drain().collect();
        for (asset_id, path) in pending_to_load {
            self.sprite_asset_manager
                .load_sprite(asset_id.clone(), path);
            self.game_state.sprite_assets.ready.push(asset_id);
        }

        // 3) 게임 로직 업데이트 (노드 트리 순회)
        self.root_node
            .update(&mut self.game_state, &self.default_global_transform);

        // 4) RenderCommand 수집
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
                    sprite_asset_id: req.sprite_asset_id.clone(),
                    descriptor_set: gpu_res.descriptor_set,
                });
            }
        }

        let device = self.device.as_ref().unwrap();
        let render_pass = self.render_pass.unwrap();
        let framebuffer = self.swapchain_framebuffers[image_index as usize];

        // 5) 인스턴스 버퍼 업로드 (렌더 패스 시작 전)
        self.sprite_renderer
            .update_instance_buffer(command_buffer, &sprite_render_commands);

        // 6) 엔진에서 렌더 패스 시작
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
            .render_pass(render_pass)
            .framebuffer(framebuffer)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: vk::Extent2D {
                    width: self.window_size.width,
                    height: self.window_size.height,
                },
            })
            .clear_values(&[vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.1, 0.1, 0.1, 1.0],
                },
            }]);

        unsafe {
            device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );
        }

        // 7) 실제 그리기 호출 (renderer.draw)
        self.sprite_renderer.draw(
            command_buffer,
            render_pass,
            framebuffer,
            viewport,
            scissor,
            &sprite_render_commands,
        );

        // 8) 엔진에서 렌더 패스 종료
        unsafe {
            device.cmd_end_render_pass(command_buffer);
        }
    }

    fn set_swapchain_framebuffers(&mut self, new_framebuffers: Vec<vk::Framebuffer>) {
        self.swapchain_framebuffers = new_framebuffers;
    }
}
