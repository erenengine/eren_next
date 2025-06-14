use std::sync::Arc;

use eren_window::window::{WindowConfig, WindowEventHandler, WindowLifecycleManager, WindowSize};
use winit::window::Window;

struct TestWindowEventHandler;

impl WindowEventHandler for TestWindowEventHandler {
    fn on_window_ready(&mut self, window: Arc<Window>) {
        println!(
            "Window ready: {}x{}",
            window.inner_size().width,
            window.inner_size().height
        );
    }

    fn on_window_lost(&mut self) {
        println!("Window lost");
    }

    fn on_window_resized(&mut self, size: WindowSize) {
        println!("Window resized: {:?}", size);
    }

    fn redraw(&mut self) {
        //println!("Redraw");
    }

    fn on_window_close_requested(&mut self) {
        println!("Window close requested");
    }
}

fn main() {
    match WindowLifecycleManager::new(
        WindowConfig {
            width: 800,
            height: 600,
            title: "Test Window",
            canvas_id: None,
        },
        TestWindowEventHandler,
    )
    .start_event_loop()
    {
        Ok(_) => {}
        Err(e) => eprintln!("Failed to start event loop: {}", e),
    }
}
