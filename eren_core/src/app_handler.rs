use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

use crate::{game::state::GameState, render::gpu::GpuContext, update::Update};

pub struct AppHandlerConfig {
    pub window_width: u32,
    pub window_height: u32,
    pub window_title: String,
}

pub struct AppHandler<T: Update> {
    config: AppHandlerConfig,
    window: Option<Arc<Window>>,
    gpu_context: Box<dyn GpuContext>,

    pub root: T,
    state: GameState,

    gpu_surface_created: bool,
}

impl<T: Update> AppHandler<T> {
    pub fn new(config: AppHandlerConfig, gpu_context: Box<dyn GpuContext>, root: T) -> Self {
        Self {
            config,
            window: None,
            gpu_context,
            root,
            state: GameState::new(),
            gpu_surface_created: false,
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
            self.window = Some(Arc::new(window));
        }

        if !self.gpu_surface_created {
            self.gpu_context
                .create_surface(self.window.as_ref().unwrap().clone());
            self.gpu_surface_created = true;
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.window = None;
        self.gpu_context.destroy_surface();
        self.gpu_surface_created = false;
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
                self.gpu_context.resize_surface(size.width, size.height);
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.root.update(&mut self.state);
                self.gpu_context.update(&mut self.state);
            }
            _ => {}
        }
    }
}
