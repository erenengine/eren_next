use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_futures::spawn_local;
use web_sys::window;

use std::{cell::RefCell, rc::Rc, sync::Arc};

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
    graphics_context: Rc<RefCell<GraphicsContext<'static, Renderer3D>>>,
    renderer: Rc<RefCell<Option<Renderer3D>>>,
}

impl WindowEventHandler for TestWindowEventHandler {
    fn on_window_ready(&mut self, window: Arc<Window>) {
        let ctx = self.graphics_context.clone();
        let renderer_handle = self.renderer.clone();
        let win = window.clone();

        spawn_local(async move {
            let init_result = ctx.borrow_mut().init(win.clone()).await;

            match init_result {
                Ok(_) => {
                    let inner_size = win.inner_size();
                    let window_size = WindowSize {
                        width: inner_size.width,
                        height: inner_size.height,
                        scale_factor: win.scale_factor(),
                    };

                    let (device_opt, format_opt) = {
                        let ctx_ref = ctx.borrow();
                        (ctx_ref.device.clone(), ctx_ref.surface_format)
                    };

                    match (device_opt, format_opt) {
                        (Some(device), Some(surface_format)) => {
                            let renderer = Renderer3D::new(&device, surface_format, window_size);
                            *renderer_handle.borrow_mut() = Some(renderer);
                        }
                        _ => show_error_popup_and_panic(
                            "Missing device or surface format after init",
                            "Renderer creation error",
                        ),
                    }
                }
                Err(e) => show_error_popup_and_panic(e, "Failed to initialise graphics context"),
            }
        });
    }

    fn on_window_lost(&mut self) {
        web_sys::console::log_1(&"Window lost".into());

        self.renderer.borrow_mut().take();
        self.graphics_context.borrow_mut().destroy();
    }

    fn on_window_resized(&mut self, size: WindowSize) {
        web_sys::console::log_1(&format!("Window resized: {:?}", size).into());

        self.graphics_context.borrow_mut().resize(size);

        if let (Some(renderer), Some(queue)) = (
            self.renderer.borrow_mut().as_mut(),
            self.graphics_context.borrow().queue.as_ref(),
        ) {
            renderer.on_window_resized(queue, size);
        }
    }

    fn redraw(&mut self) {
        if let Some(renderer) = self.renderer.borrow().as_ref() {
            if let Err(e) = self.graphics_context.borrow_mut().redraw(renderer) {
                show_error_popup_and_panic(e, "Failed to redraw");
            }
        }
    }

    fn on_window_close_requested(&mut self) {
        self.renderer.borrow_mut().take();
        self.graphics_context.borrow_mut().destroy();
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
            graphics_context: Rc::new(RefCell::new(GraphicsContext::new())),
            renderer: Rc::new(RefCell::new(None)),
        },
    )
    .start_event_loop()
    {
        Ok(_) => {}
        Err(e) => show_error_popup_and_panic(e, "Failed to start event loop"),
    }
}
