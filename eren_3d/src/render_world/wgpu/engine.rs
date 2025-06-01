use std::{hash::Hash, time::Instant};

use eren_core::render_world::wgpu::engine::WgpuEngine;

use crate::{
    game_world::{nodes::game_node::GameNode, state::GameState, transform::GlobalTransform},
    render_world::wgpu::renderers::model_renderer::ModelRenderCommand,
};

use winit::dpi::PhysicalSize;

use super::{
    asset_managers::model_asset_manager::WgpuModelAssetManager,
    bind_group_layout::create_material_bind_group_layout::create_material_bind_group_layout,
    renderers::model_renderer::{CameraUniform, WgpuModelRenderer},
};

pub struct WgpuEngine3D<R, MA> {
    game_state: GameState<MA>,
    root_node: R,
    default_global_transform: GlobalTransform,

    model_asset_manager: WgpuModelAssetManager<MA>,
    model_renderer: WgpuModelRenderer<MA>,

    last_frame_time: Instant,
}

impl<R, MA> WgpuEngine3D<R, MA>
where
    R: GameNode<MA>,
    MA: Eq + Hash + Copy,
{
    pub fn new(root_node: R) -> Self {
        Self {
            game_state: GameState::new(),
            root_node,
            default_global_transform: GlobalTransform::new(),

            model_asset_manager: WgpuModelAssetManager::new(),
            model_renderer: WgpuModelRenderer::new(),

            last_frame_time: Instant::now(),
        }
    }
}

impl<R, MA> WgpuEngine for WgpuEngine3D<R, MA>
where
    R: GameNode<MA>,
    MA: Eq + Hash + Copy,
{
    fn on_gpu_resources_ready(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
        window_size: PhysicalSize<u32>,
        window_scale_factor: f64,
    ) {
        let material_bind_group_layout = create_material_bind_group_layout(device);
        self.model_asset_manager
            .on_gpu_resources_ready(device, queue, &material_bind_group_layout);

        self.model_renderer.on_gpu_resources_ready(
            device,
            queue,
            surface_format,
            &material_bind_group_layout,
            window_size,
            window_scale_factor,
        );
    }

    fn on_gpu_resources_lost(&mut self) {
        self.model_asset_manager.on_gpu_resources_lost();
        self.model_renderer.on_gpu_resources_lost();
    }

    fn on_window_resized(&mut self, window_size: PhysicalSize<u32>, window_scale_factor: f64) {
        self.game_state.window_size = window_size;
        self.model_renderer
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

        for (asset, path) in self.game_state.model_assets.pending.drain() {
            self.model_asset_manager.load_model(asset, path);
            self.game_state.model_assets.ready.push(asset);
        }

        self.root_node
            .update(&mut self.game_state, &self.default_global_transform);

        let mut render_commands: Vec<ModelRenderCommand<MA>> = vec![];
        for render_request in self.game_state.render_requests.drain(..) {
            let asset_id = render_request.model_asset_id;
            let gpu_resource = self.model_asset_manager.get_gpu_resource(asset_id);

            if let Some(gpu_resource) = gpu_resource {
                render_commands.push(ModelRenderCommand {
                    matrix: render_request.matrix,
                    alpha: render_request.alpha,
                    material_asset_id: asset_id,
                    model_gpu_resource: gpu_resource.clone(),
                });
            }
        }

        self.model_renderer
            .render(surface_texture_view, command_encoder, render_commands);
    }
}
