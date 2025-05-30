use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

use crate::{
    render::{GraphicsLibrary, gpu::GpuState},
    update::Update,
};

pub struct AppHandlerConfig {
    pub window_width: u32,
    pub window_height: u32,
    pub window_title: String,
    pub graphics_library: GraphicsLibrary,
}

pub struct AppHandler<T: Update> {
    config: AppHandlerConfig,
    window: Option<Window>,
    gpu: Box<dyn GpuState>,
    pub root: T,

    gpu_initialized: bool,
}

impl<T: Update> AppHandler<T> {
    pub fn new(config: AppHandlerConfig, root: T) -> Self {
        let gpu: Box<dyn GpuState>;
        if config.graphics_library == GraphicsLibrary::Wgpu {
            gpu = Box::new(crate::render::wgpu::gpu::WgpuGpuState::new());
        } else {
            gpu = Box::new(crate::render::ash::gpu::AshGpuState::new());
        }

        Self {
            config,
            window: None,
            gpu,
            root,
            gpu_initialized: false,
        }
    }

    pub fn run(&mut self) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(self).unwrap();
    }
}

impl<T: Update> ApplicationHandler for AppHandler<T> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let attrs = Window::default_attributes()
                .with_title(self.config.window_title.clone())
                .with_inner_size(LogicalSize::new(
                    self.config.window_width,
                    self.config.window_height,
                ));
            let window = event_loop.create_window(attrs).unwrap();
            self.window = Some(window);
        }

        if !self.gpu_initialized {
            self.gpu.init(&self.window.as_ref().unwrap());
            self.gpu_initialized = true;
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.window = None;
        self.gpu.cleanup();
        self.gpu_initialized = false;
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: winit::event::StartCause) {
        if let Some(window) = &self.window {
            if let StartCause::Poll = cause {
                window.request_redraw();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::Resized(size) => {
                self.gpu.resize_surface(size.width, size.height);
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.gpu.draw_frame();
            }
            _ => {}
        }
    }
}
