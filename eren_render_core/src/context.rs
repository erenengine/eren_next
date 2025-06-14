use std::sync::Arc;
use thiserror::Error;

use eren_window::window::WindowSize;
use wgpu::util::new_instance_with_webgpu_detection;
use winit::window::Window;

use crate::renderer::{FrameContext, Renderer};

#[derive(Debug, Error)]
pub enum GraphicsContextError {
    #[error("Failed to create surface: {0}")]
    CreateSurface(#[from] wgpu::CreateSurfaceError),

    #[error("Failed to request adapter: {0}")]
    RequestAdapter(#[from] wgpu::RequestAdapterError),

    #[error("Failed to request device: {0}")]
    RequestDevice(#[from] wgpu::RequestDeviceError),
}

pub struct GraphicsContext<'a, R: Renderer> {
    instance: Option<wgpu::Instance>,

    pub device: Option<wgpu::Device>,
    pub queue: Option<wgpu::Queue>,

    surface: Option<wgpu::Surface<'a>>,
    pub surface_format: Option<wgpu::TextureFormat>,
    surface_config: Option<wgpu::SurfaceConfiguration>,

    phantom: std::marker::PhantomData<R>,
}

impl<'a, R: Renderer> GraphicsContext<'a, R> {
    pub fn new() -> Self {
        Self {
            instance: None,

            device: None,
            queue: None,
            surface: None,
            surface_format: None,
            surface_config: None,

            phantom: std::marker::PhantomData,
        }
    }

    pub async fn init(&mut self, window: Arc<Window>) -> Result<(), GraphicsContextError> {
        let instance_desc = wgpu::InstanceDescriptor {
            backends: wgpu::Backends::GL, // WebGL 대상
            ..Default::default()
        };

        let instance = new_instance_with_webgpu_detection(&instance_desc).await;

        let surface = instance.create_surface(window.clone())?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await?;

        let limits = adapter.limits();

        let mut request_device_desc = wgpu::DeviceDescriptor::default();
        request_device_desc.required_limits = limits;

        let (device, queue) = adapter.request_device(&request_device_desc).await?;

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let window_size = window.inner_size();
        let scale_factor = window.scale_factor();

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window_size.width / scale_factor as u32,
            height: window_size.height / scale_factor as u32,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_config);

        self.instance = Some(instance);
        self.device = Some(device);
        self.queue = Some(queue);
        self.surface = Some(surface);
        self.surface_format = Some(surface_format);
        self.surface_config = Some(surface_config);

        Ok(())
    }

    pub fn resize(&mut self, window_size: WindowSize) {
        if let (Some(device), Some(surface), Some(surface_config)) =
            (&self.device, &self.surface, &mut self.surface_config)
        {
            surface_config.width = window_size.width;
            surface_config.height = window_size.height;
            surface.configure(device, surface_config);
        }
    }

    pub fn destroy(&mut self) {
        self.surface_config = None;
        self.surface_format = None;
        self.surface = None;
        self.queue = None;
        self.device = None;
    }

    pub fn redraw(&mut self, renderer: &R) -> Result<(), wgpu::SurfaceError> {
        if let (Some(device), Some(queue), Some(surface)) =
            (&self.device, &self.queue, &self.surface)
        {
            let output = surface.get_current_texture()?;
            let view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

            renderer.render(&mut FrameContext {
                view: &view,
                encoder: &mut encoder,
            });

            queue.submit(std::iter::once(encoder.finish()));
            output.present();
        }

        Ok(())
    }
}
