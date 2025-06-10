use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
    pub title: &'static str,
}

#[derive(Copy, Clone, PartialEq)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
    pub scale_factor: f64,
}

pub trait WindowEventHandler {
    fn on_window_ready(&mut self, window: &Window);
    fn on_window_lost(&mut self);
    fn on_window_resized(&mut self, size: WindowSize);
    fn redraw(&mut self);
}

pub struct WindowLifecycleManager<E: WindowEventHandler> {
    config: WindowConfig,
    event_handler: E,
    window: Option<Window>,
    current_window_size: Option<WindowSize>,
}

impl<E: WindowEventHandler> WindowLifecycleManager<E> {
    pub fn new(config: WindowConfig, event_handler: E) -> Self {
        Self {
            config,
            event_handler,
            window: None,
            current_window_size: None,
        }
    }

    fn handle_resize_event(&mut self, new_size: WindowSize) {
        if self.current_window_size != Some(new_size) {
            self.current_window_size = Some(new_size);
            self.event_handler.on_window_resized(new_size);
        }
    }

    pub fn start_event_loop(&mut self) {
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(self).unwrap();
    }
}

impl<E: WindowEventHandler> ApplicationHandler for WindowLifecycleManager<E> {
    fn new_events(&mut self, _: &ActiveEventLoop, cause: StartCause) {
        if let Some(window) = &self.window {
            if let StartCause::Poll = cause {
                window.request_redraw();
            }
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            let window = event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title(self.config.title)
                        .with_inner_size(LogicalSize::new(self.config.width, self.config.height)),
                )
                .unwrap();

            self.event_handler.on_window_ready(&window);

            self.window = Some(window);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::Resized(size) => {
                if let Some(window) = &self.window {
                    self.handle_resize_event(WindowSize {
                        width: size.width,
                        height: size.height,
                        scale_factor: window.scale_factor(),
                    });
                }
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::ScaleFactorChanged { scale_factor, .. } => {
                if let Some(window) = &self.window {
                    let inner = window.inner_size();
                    self.handle_resize_event(WindowSize {
                        width: inner.width,
                        height: inner.height,
                        scale_factor,
                    });
                }
            }
            WindowEvent::RedrawRequested => {
                self.event_handler.redraw();
            }
            _ => {}
        }
    }

    fn suspended(&mut self, _: &ActiveEventLoop) {
        self.window = None;
        self.event_handler.on_window_lost();
    }
}
