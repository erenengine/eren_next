use std::sync::Arc;

use eren_window::window::{WindowConfig, WindowEventHandler, WindowLifecycleManager, WindowSize};
use wasm_bindgen::prelude::wasm_bindgen;
use winit::window::Window;

struct TestWindowEventHandler;

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
    }

    fn on_window_lost(&mut self) {
        web_sys::console::log_1(&"Window lost".into());
    }

    fn on_window_resized(&mut self, size: WindowSize) {
        web_sys::console::log_1(&format!("Window resized: {:?}", size).into());
    }

    fn redraw(&mut self) {
        //web_sys::console::log_1(&"Redraw".into());
    }

    fn on_window_close_requested(&mut self) {
        web_sys::console::log_1(&"Window close requested".into());
    }
}

fn main() {}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() {
    WindowLifecycleManager::new(
        WindowConfig {
            canvas_id: "canvas",
        },
        TestWindowEventHandler,
    )
    .start_event_loop();
}
