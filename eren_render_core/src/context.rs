use std::sync::Arc;

use eren_window::window::WindowSize;
use winit::window::Window;

#[derive(Debug)]
pub struct FrameContext<'a> {
    pub view: &'a wgpu::TextureView,
    pub encoder: &'a mut wgpu::CommandEncoder,
}

pub struct GraphicsContext<'a, F>
where
    F: Fn(&FrameContext),
{
    draw_frame: F,
    instance: wgpu::Instance,

    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    surface: Option<wgpu::Surface<'a>>,
    surface_config: Option<wgpu::SurfaceConfiguration>,
}

impl<'a, F> GraphicsContext<'a, F>
where
    F: Fn(&FrameContext),
{
    pub fn new(draw_frame: F) -> Self {
        Self {
            draw_frame,
            instance: wgpu::Instance::default(),

            device: None,
            queue: None,
            surface: None,
            surface_config: None,
        }
    }

    pub async fn init(&mut self, window: Arc<Window>) {
        let surface = self
            .instance
            .create_surface(window.clone())
            .expect("Failed to create surface");

        let adapter = self
            .instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("Failed to request adapter");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .expect("Failed to request device");

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let window_size = window.inner_size();

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: window_size.width,
            height: window_size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_config);

        self.device = Some(device);
        self.queue = Some(queue);
        self.surface = Some(surface);
        self.surface_config = Some(surface_config);
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
        self.surface = None;
        self.surface_config = None;
        self.device = None;
        self.queue = None;
    }

    pub fn redraw(&mut self) -> Result<(), wgpu::SurfaceError> {
        if let (Some(device), Some(queue), Some(surface)) =
            (&self.device, &self.queue, &self.surface)
        {
            let output = surface.get_current_texture()?;
            let view = output
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            let mut encoder =
                device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

            (self.draw_frame)(&FrameContext {
                view: &view,
                encoder: &mut encoder,
            });

            queue.submit(std::iter::once(encoder.finish()));
            output.present();
        }

        Ok(())
    }
}
