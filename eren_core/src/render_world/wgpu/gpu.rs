use std::sync::Arc;

use super::engine::WgpuEngine;
use crate::render_world::common::gpu::GpuResourceManager;
use winit::window::Window;

pub struct WgpuGpuResourceManager {
    engine: Box<dyn WgpuEngine>,
    instance: wgpu::Instance,

    surface: Option<wgpu::Surface<'static>>,
    surface_config: Option<wgpu::SurfaceConfiguration>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
}

impl WgpuGpuResourceManager {
    pub fn new(engine: Box<dyn WgpuEngine>) -> Self {
        Self {
            engine,
            instance: wgpu::Instance::default(),

            surface: None,
            surface_config: None,
            device: None,
            queue: None,
        }
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

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: vec![],
            desired_maximum_frame_latency: 1,
        };

        surface.configure(&device, &surface_config);

        self.surface = Some(surface);
        self.surface_config = Some(surface_config);
        self.device = Some(device.clone());
        self.queue = Some(queue.clone());

        self.engine.on_gpu_resources_ready(&device, &queue);
    }
}

impl GpuResourceManager for WgpuGpuResourceManager {
    fn on_window_ready(&mut self, window: Arc<Window>) {
        pollster::block_on(self.init_gpu(window));
    }

    fn on_window_lost(&mut self) {
        self.surface = None;
        self.surface_config = None;
        self.device = None;
        self.queue = None;

        self.engine.on_gpu_resources_lost();
    }

    fn on_window_resized(&mut self, width: u32, height: u32) {
        if let (Some(surface), Some(device), Some(surface_config)) =
            (&self.surface, &self.device, &mut self.surface_config)
        {
            surface_config.width = width;
            surface_config.height = height;
            surface.configure(device, surface_config);
        }
    }

    fn update(&mut self) {
        if let (Some(surface), Some(device), Some(queue)) =
            (&self.surface, &self.device, &self.queue)
        {
            let surface_texture = surface.get_current_texture().unwrap();
            let surface_texture_view = surface_texture
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let mut command_encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

            self.engine
                .update(&surface_texture_view, &mut command_encoder);

            queue.submit(Some(command_encoder.finish()));
            surface_texture.present();
        }
    }
}
