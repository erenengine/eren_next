use winit::{dpi::LogicalSize, event_loop::ActiveEventLoop, window::Window};

pub struct GpuState<'a> {
    pub instance: wgpu::Instance,
    pub window: Option<&'a Window>,
    pub surface: Option<wgpu::Surface<'a>>,
    pub surface_config: Option<wgpu::SurfaceConfiguration>,
    pub device: Option<wgpu::Device>,
    pub queue: Option<wgpu::Queue>,
    pub gpu_initialized: bool,
}

impl<'a> GpuState<'a> {
    pub fn new() -> Self {
        Self {
            instance: wgpu::Instance::default(),
            window: None,
            surface: None,
            surface_config: None,
            device: None,
            queue: None,
            gpu_initialized: false,
        }
    }

    pub fn create_window_if_needed(
        &mut self,
        event_loop: &ActiveEventLoop,
        title: &str,
        width: u32,
        height: u32,
    ) {
        if self.window.is_none() {
            let attrs = Window::default_attributes()
                .with_title(title.to_string())
                .with_inner_size(LogicalSize::new(width, height));
            let window = event_loop.create_window(attrs).unwrap();
            self.window = Some(Box::leak(Box::new(window)));
        }
    }

    pub fn ensure_initialized(&mut self) {
        if !self.gpu_initialized {
            pollster::block_on(self.init_gpu());
            self.gpu_initialized = true;
        }
    }

    pub fn release(&mut self) {
        self.window = None;
        self.surface = None;
        self.surface_config = None;
        self.device = None;
        self.queue = None;
        self.gpu_initialized = false;
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

    pub fn resize_surface(&mut self, width: u32, height: u32) {
        if let (Some(surface), Some(device), Some(config)) =
            (&self.surface, &self.device, &mut self.surface_config)
        {
            config.width = width;
            config.height = height;
            surface.configure(device, config);
        }
    }
}
