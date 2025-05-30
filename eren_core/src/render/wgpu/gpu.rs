use std::sync::Arc;

use crate::{asset::AssetManager, game::state::GameState, render::gpu::GpuContext};
use winit::window::Window;

use super::{asset::WgpuAssetManager, pass::WgpuRenderPass};

pub struct WgpuGpuContext {
    pub asset_manager: WgpuAssetManager,
    render_passes: Vec<Box<dyn WgpuRenderPass>>,

    instance: wgpu::Instance,
    surface: Option<wgpu::Surface<'static>>,
    surface_config: Option<wgpu::SurfaceConfiguration>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
}

impl WgpuGpuContext {
    pub fn new() -> Self {
        Self {
            asset_manager: WgpuAssetManager::new(),
            render_passes: Vec::new(),

            instance: wgpu::Instance::default(),
            surface: None,
            surface_config: None,
            device: None,
            queue: None,
        }
    }

    pub fn add_render_pass(&mut self, render_pass: Box<dyn WgpuRenderPass>) {
        self.render_passes.push(render_pass);
    }

    async fn init_gpu(&mut self, window: Arc<Window>) {
        let surface = self.instance.create_surface(window.clone()).unwrap();

        let adapter = self
            .instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let format = surface_caps.formats[0];
        let size = window.inner_size();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
            desired_maximum_frame_latency: 1,
        };

        surface.configure(&device, &config);

        self.surface = Some(surface);
        self.surface_config = Some(config);
        self.device = Some(device.clone());
        self.queue = Some(queue.clone());

        self.asset_manager.surface_created(&device, &queue);

        for render_pass in &mut self.render_passes {
            render_pass.surface_created();
        }
    }
}

impl GpuContext for WgpuGpuContext {
    fn create_surface(&mut self, window: Arc<Window>) {
        pollster::block_on(self.init_gpu(window));
    }

    fn destroy_surface(&mut self) {
        self.surface = None;
        self.surface_config = None;
        self.device = None;
        self.queue = None;

        self.asset_manager.surface_destroyed();

        for render_pass in &mut self.render_passes {
            render_pass.surface_destroyed();
        }
    }

    fn resize_surface(&mut self, width: u32, height: u32) {
        if let (Some(surface), Some(device), Some(config)) =
            (&self.surface, &self.device, &mut self.surface_config)
        {
            config.width = width;
            config.height = height;
            surface.configure(device, config);

            for render_pass in &mut self.render_passes {
                render_pass.window_resized();
            }
        }
    }

    fn update(&mut self, state: &mut GameState) {
        if let (Some(surface), Some(device), Some(queue)) =
            (&self.surface, &self.device, &self.queue)
        {
            let frame = surface.get_current_texture().unwrap();
            let view = frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

            self.asset_manager.ensure_asset_loaded(state);

            for render_pass in &mut self.render_passes {
                render_pass.render(&mut encoder, &view);
            }

            queue.submit(Some(encoder.finish()));
            frame.present();
        }
    }
}
