use std::sync::Arc;

use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

use crate::render_world::common::gpu::GpuResourceManager;

pub struct WindowConfig {
    pub window_width: u32,
    pub window_height: u32,
    pub window_title: String,
}

pub struct WindowLifecycleManager {
    config: WindowConfig,
    gpu_resource_manager: Box<dyn GpuResourceManager>,
    window: Option<Arc<Window>>,
}

impl WindowLifecycleManager {
    pub fn new(config: WindowConfig, gpu_resource_manager: Box<dyn GpuResourceManager>) -> Self {
        Self {
            config,
            gpu_resource_manager,
            window: None,
        }
    }

    pub fn run(&mut self) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(self).unwrap();
    }
}

impl ApplicationHandler for WindowLifecycleManager {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let attributes = Window::default_attributes()
                .with_title(self.config.window_title.clone())
                .with_inner_size(LogicalSize::new(
                    self.config.window_width,
                    self.config.window_height,
                ));
            let window = Arc::new(event_loop.create_window(attributes).unwrap());
            self.window = Some(window.clone());
            self.gpu_resource_manager.on_window_ready(window);
        }
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        self.window = None;
        self.gpu_resource_manager.on_window_lost();
    }

    fn new_events(&mut self, _event_loop: &ActiveEventLoop, cause: StartCause) {
        if let Some(window) = &self.window {
            if let StartCause::Poll = cause {
                window.request_redraw();
            }
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::Resized(size) => {
                self.gpu_resource_manager
                    .on_window_resized(size.width, size.height);
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.gpu_resource_manager.update();
            }
            _ => {}
        }
    }
}
