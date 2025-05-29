use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

use crate::{asset::AssetManager, context::GameContext, core::Updatable};

pub struct App<'a, T: Updatable> {
    window_width: u32,
    window_height: u32,
    window_title: String,

    instance: wgpu::Instance,
    window: Option<&'a Window>,
    surface: Option<wgpu::Surface<'a>>,
    surface_config: Option<wgpu::SurfaceConfiguration>,
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    gpu_initialized: bool,

    context: GameContext,
    pub asset_manager: AssetManager,
    pub root: T,
}

impl<'a, T: Updatable> App<'a, T> {
    pub fn new(window_width: u32, window_height: u32, window_title: &str, root: T) -> Self {
        Self {
            window_width,
            window_height,
            window_title: window_title.into(),

            instance: wgpu::Instance::default(),
            device: None,
            queue: None,
            window: None,
            surface: None,
            surface_config: None,
            gpu_initialized: false,

            context: GameContext::new(),
            asset_manager: AssetManager::new(),
            root,
        }
    }

    async fn init_gpu(&mut self) {
        let window = self.window.unwrap();
        let surface = self.instance.create_surface(window).unwrap();

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
        self.device = Some(device);
        self.queue = Some(queue);
    }

    fn resize_surface(&mut self, new_width: u32, new_height: u32) {
        if let (Some(surface), Some(device), Some(config)) =
            (&self.surface, &self.device, &mut self.surface_config)
        {
            config.width = new_width;
            config.height = new_height;
            surface.configure(device, config);
        }
    }

    fn draw_frame(&mut self) {
        let device = match self.device.as_ref() {
            Some(device) => device,
            None => return,
        };
        let queue = match self.queue.as_ref() {
            Some(queue) => queue,
            None => return,
        };
        let surface = match self.surface.as_ref() {
            Some(surface) => surface,
            None => return,
        };

        let frame = match surface.get_current_texture() {
            Ok(frame) => frame,
            Err(e) => {
                eprintln!("Failed to acquire frame: {e}");
                return;
            }
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let _rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            self.root.update(&mut self.context);
        }

        queue.submit(Some(encoder.finish()));
        frame.present();
    }

    pub fn run(&mut self) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(self).unwrap();
    }
}

impl<'a, T: Updatable> ApplicationHandler for App<'a, T> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let attrs = Window::default_attributes()
                .with_title(self.window_title.clone())
                .with_inner_size(LogicalSize::new(self.window_width, self.window_height));
            let window = event_loop.create_window(attrs).unwrap();
            self.window = Some(Box::leak(Box::new(window)));
        }

        if !self.gpu_initialized {
            pollster::block_on(self.init_gpu());
            self.gpu_initialized = true;
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.window = None;
        self.surface = None;
        self.surface_config = None;
        self.device = None;
        self.queue = None;
        self.gpu_initialized = false;
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: winit::event::StartCause) {
        if let Some(window) = self.window {
            if let StartCause::Poll = cause {
                window.request_redraw();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::Resized(size) => {
                self.resize_surface(size.width, size.height);
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.draw_frame();
            }
            _ => {}
        }
    }
}
