use std::sync::Arc;

use eren_render_core::context::{FrameContext, GraphicsContext};
use eren_window::window::{WindowConfig, WindowEventHandler, WindowLifecycleManager, WindowSize};
use winit::window::Window;

struct TestWindowEventHandler<F>
where
    F: Fn(&FrameContext),
{
    graphics_context: GraphicsContext<F>,
}

impl<F> WindowEventHandler for TestWindowEventHandler<F>
where
    F: Fn(&FrameContext),
{
    fn on_window_ready(&mut self, window: Arc<Window>) {
        println!(
            "Window ready: {}x{}",
            window.inner_size().width,
            window.inner_size().height
        );

        self.graphics_context.init(&window);
    }

    fn on_window_lost(&mut self) {
        println!("Window lost");

        self.graphics_context.destroy();
    }

    fn on_window_resized(&mut self, size: WindowSize) {
        println!("Window resized: {:?}", size);

        self.graphics_context.resize(size);
    }

    fn redraw(&mut self) {
        //println!("Redraw");

        self.graphics_context.redraw();
    }
}

fn main() {
    let draw_frame = |frame_context: &FrameContext| {
        println!("Draw frame: {:?}", frame_context);
    };

    WindowLifecycleManager::new(
        WindowConfig {
            width: 800,
            height: 600,
            title: "Test Window",
        },
        TestWindowEventHandler {
            graphics_context: GraphicsContext::new(draw_frame),
        },
    )
    .start_event_loop();
}
