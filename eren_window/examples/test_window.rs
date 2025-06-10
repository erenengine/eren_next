use eren_window::window::{WindowConfig, WindowEventHandler, WindowLifecycleManager, WindowSize};
use winit::window::Window;

struct TestWindowEventHandler;

impl WindowEventHandler for TestWindowEventHandler {
    fn on_window_ready(&mut self, window: &Window) {
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
        println!("Window resized: {}x{}", size.width, size.height);
    }

    fn redraw(&mut self) {
        //println!("Redraw");
    }
}

fn main() {
    WindowLifecycleManager::new(
        WindowConfig {
            width: 800,
            height: 600,
            title: "Test Window",
        },
        TestWindowEventHandler,
    )
    .start_event_loop();
}
