use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};

pub struct WindowConfig {
    pub window_width: u32,
    pub window_height: u32,
    pub window_title: &'static str,
}

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
}

impl<E: WindowEventHandler> WindowLifecycleManager<E> {
    pub fn new(config: WindowConfig, event_handler: E) -> Self {
        Self {
            config,
            event_handler,
            window: None,
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
                        .with_title(self.config.window_title)
                        .with_inner_size(LogicalSize::new(
                            self.config.window_width,
                            self.config.window_height,
                        )),
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
                    self.event_handler.on_window_resized(WindowSize {
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
                    self.event_handler.on_window_resized(WindowSize {
                        width: window.inner_size().width,
                        height: window.inner_size().height,
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
