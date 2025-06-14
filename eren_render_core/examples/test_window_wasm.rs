use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen_futures::spawn_local;
use web_sys::window;

use std::{cell::RefCell, rc::Rc, sync::Arc};

use eren_render_core::{
    context::GraphicsContext,
    renderer::{FrameContext, Renderer},
};
use eren_window::window::{WindowConfig, WindowEventHandler, WindowLifecycleManager, WindowSize};
use winit::window::Window;

pub fn show_error_popup_and_panic<E: std::fmt::Display>(error: E, context: &str) -> ! {
    let window = window().expect("no global `window` exists");
    window
        .alert_with_message(&format!("{}: {}", context, error))
        .unwrap();

    panic!("{}: {}", context, error);
}

struct EmptyRenderer;

impl Renderer for EmptyRenderer {
    fn render<'a>(&self, _frame_context: &mut FrameContext<'a>) {}
}

struct TestWindowEventHandler {
    graphics_context: Rc<RefCell<GraphicsContext<'static, EmptyRenderer>>>,
    renderer: Rc<RefCell<Option<EmptyRenderer>>>,
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

        let ctx = self.graphics_context.clone();
        let renderer_handle = self.renderer.clone();

        spawn_local(async move {
            match ctx.borrow_mut().init(window).await {
                Ok(_) => *renderer_handle.borrow_mut() = Some(EmptyRenderer),
                Err(e) => show_error_popup_and_panic(e, "Failed to initialize graphics context"),
            };
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
    WindowLifecycleManager::new(
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
    .start_event_loop();
}
