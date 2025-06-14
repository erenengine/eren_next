use std::sync::Arc;

use thiserror::Error;
use winit::{
    application::ApplicationHandler,
    error::EventLoopError,
    event::{StartCause, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    window::{Window, WindowId},
};

#[cfg(not(target_arch = "wasm32"))]
use winit::{dpi::LogicalSize, event_loop::ControlFlow};

#[cfg(target_arch = "wasm32")]
use {
    wasm_bindgen::JsCast,
    web_sys::HtmlCanvasElement,
    winit::platform::web::{EventLoopExtWebSys, WindowAttributesExtWebSys},
};

#[derive(Debug, Error)]
pub enum WindowLifecycleManagerError {
    #[error("Event loop error: {0}")]
    EventLoopError(#[from] EventLoopError),
}

pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
    pub title: &'static str,
    pub canvas_id: Option<&'static str>,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
    pub scale_factor: f64,
}

pub trait WindowEventHandler {
    fn on_window_ready(&mut self, window: Arc<Window>);
    fn on_window_lost(&mut self);
    fn on_window_resized(&mut self, size: WindowSize);
    fn redraw(&mut self);
    fn on_window_close_requested(&mut self);
}

pub struct WindowLifecycleManager<E: WindowEventHandler> {
    config: WindowConfig,
    event_handler: E,
    window: Option<Arc<Window>>,
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
        // (0,0)은 일부 플랫폼에서 초기화 과정에서 나오는 값이므로 무시
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }

        if self.current_window_size != Some(new_size) {
            self.current_window_size = Some(new_size);
            self.event_handler.on_window_resized(new_size);
        }
    }
}

impl<E> WindowLifecycleManager<E>
where
    E: WindowEventHandler + 'static,
{
    #[cfg(target_arch = "wasm32")]
    pub fn start_event_loop(self) -> Result<(), WindowLifecycleManagerError> {
        let event_loop = EventLoop::new()?;
        event_loop.spawn_app(self);
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn start_event_loop(&mut self) -> Result<(), WindowLifecycleManagerError> {
        let event_loop = EventLoop::new()?;
        event_loop.set_control_flow(ControlFlow::Poll);
        event_loop.run_app(self)?;
        Ok(())
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
        if self.window.is_some() {
            return;
        }

        let raw_window = {
            #[cfg(not(target_arch = "wasm32"))]
            {
                event_loop
                    .create_window(
                        Window::default_attributes()
                            .with_title(self.config.title)
                            .with_inner_size(LogicalSize::new(
                                self.config.width,
                                self.config.height,
                            )),
                    )
                    .expect("Failed to create native window")
            }

            #[cfg(target_arch = "wasm32")]
            {
                // Look up the target <canvas> from the HTML document.
                let canvas: HtmlCanvasElement = {
                    let window = web_sys::window().expect("No global `window`");
                    let document = window.document().expect("No Document");
                    document
                        .get_element_by_id(self.config.canvas_id.expect("Canvas ID is not set"))
                        .unwrap_or_else(|| {
                            panic!(
                                "Canvas element #{} not found",
                                self.config.canvas_id.expect("Canvas ID is not set")
                            )
                        })
                        .dyn_into::<HtmlCanvasElement>()
                        .expect("Element is not a canvas")
                };

                event_loop
                    .create_window(Window::default_attributes().with_canvas(Some(canvas)))
                    .expect("Failed to create web window")
            }
        };

        let window = Arc::new(raw_window);

        self.event_handler.on_window_ready(window.clone());
        self.window = Some(window);
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
                self.event_handler.on_window_close_requested();
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

                #[cfg(target_arch = "wasm32")]
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn suspended(&mut self, _: &ActiveEventLoop) {
        self.window = None;
        self.event_handler.on_window_lost();
    }
}
