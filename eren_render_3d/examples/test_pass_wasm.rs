use futures::lock::Mutex;

use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_futures::spawn_local;
use web_sys::window;

use std::sync::Arc;

use eren_render_3d::renderer::Renderer3D;
use eren_render_core::context::GraphicsContext;
use eren_window::window::{WindowConfig, WindowEventHandler, WindowLifecycleManager, WindowSize};
use winit::window::Window;

pub fn show_error_popup_and_panic<E: std::fmt::Display>(error: E, context: &str) -> ! {
    let window = window().expect("no global `window` exists");
    window
        .alert_with_message(&format!("{}: {}", context, error))
        .unwrap();

    panic!("{}: {}", context, error);
}

struct TestWindowEventHandler {
    initialized: bool,
    window: Option<Arc<Window>>,
    graphics_context: Arc<Mutex<GraphicsContext<'static, Renderer3D>>>,
    renderer: Arc<Mutex<Option<Renderer3D>>>,
}

impl TestWindowEventHandler {
    fn initialize(&mut self) {
        self.initialized = true;

        let ctx = self.graphics_context.clone();
        let renderer_handle: Arc<Mutex<Option<Renderer3D>>> = self.renderer.clone();
        let win = self.window.clone().unwrap();

        spawn_local(async move {
            let mut ctx_lock = ctx.lock().await;
            if let Err(e) = ctx_lock.init(win.clone()).await {
                show_error_popup_and_panic(e, "Failed to initialize graphics context");
            }

            web_sys::console::log_1(&"Create renderer".into());

            let inner_size = win.inner_size();
            let window_size = WindowSize {
                width: inner_size.width,
                height: inner_size.height,
                scale_factor: win.scale_factor(),
            };

            let device = ctx_lock.device.clone().unwrap();
            let surface_format = ctx_lock.surface_format.clone().unwrap();

            let mut renderer_handle_lock = renderer_handle.lock().await;
            *renderer_handle_lock = Some(Renderer3D::new(&device, surface_format, window_size));
        });
    }
}

impl WindowEventHandler for TestWindowEventHandler {
    fn on_window_ready(&mut self, window: Arc<Window>) {
        web_sys::console::log_1(
            &format!(
                "Window ready: {}x{}",
                window.inner_size().width,
                window.inner_size().height
            )
            .into(),
        );

        self.window = Some(window.clone());

        if window.inner_size().width > 0 && window.inner_size().height > 0 {
            self.initialize();
        }
    }

    fn on_window_lost(&mut self) {
        web_sys::console::log_1(&"Window lost".into());

        let renderer = self.renderer.clone();
        let ctx = self.graphics_context.clone();

        spawn_local(async move {
            let mut renderer_lock = renderer.lock().await;
            let mut ctx_lock = ctx.lock().await;

            renderer_lock.take();
            ctx_lock.destroy();
        });
    }

    fn on_window_resized(&mut self, size: WindowSize) {
        web_sys::console::log_1(&format!("Window resized: {:?}", size).into());

        if self.initialized {
            let ctx = self.graphics_context.clone();

            spawn_local(async move {
                web_sys::console::log_1(&"Resize".into());

                let mut ctx_lock = ctx.lock().await;
                ctx_lock.resize(size);
            });
        } else if size.width > 0 && size.height > 0 {
            self.initialize();
        }
    }

    fn redraw(&mut self) {
        //web_sys::console::log_1(&"Redraw".into());

        let renderer = self.renderer.clone();
        let ctx = self.graphics_context.clone();

        spawn_local(async move {
            let renderer_lock = renderer.lock().await;
            if let Some(renderer) = renderer_lock.as_ref() {
                let mut ctx_lock = ctx.lock().await;
                if let Err(e) = ctx_lock.redraw(renderer) {
                    show_error_popup_and_panic(e, "Failed to redraw");
                }
            }
        });
    }

    fn on_window_close_requested(&mut self) {
        let renderer = self.renderer.clone();
        let ctx = self.graphics_context.clone();

        spawn_local(async move {
            let mut renderer_lock = renderer.lock().await;
            let mut ctx_lock = ctx.lock().await;

            renderer_lock.take();
            ctx_lock.destroy();
        });
    }
}

fn main() {}

#[wasm_bindgen(start)]
pub fn start() {
    match WindowLifecycleManager::new(
        WindowConfig {
            width: 800,
            height: 600,
            title: "Test Window",
            canvas_id: Some("canvas"),
        },
        TestWindowEventHandler {
            initialized: false,
            window: None,
            graphics_context: Arc::new(Mutex::new(GraphicsContext::new())),
            renderer: Arc::new(Mutex::new(None)),
        },
    )
    .start_event_loop()
    {
        Ok(_) => {}
        Err(e) => show_error_popup_and_panic(e, "Failed to start event loop"),
    }
}
