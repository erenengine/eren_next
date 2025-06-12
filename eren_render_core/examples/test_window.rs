use std::sync::Arc;

use eren_render_core::context::{FrameContext, GraphicsContext};
use eren_window::{
    error::show_error_popup_and_panic,
    window::{WindowConfig, WindowEventHandler, WindowLifecycleManager, WindowSize},
};
use winit::window::Window;

struct TestWindowEventHandler<'a, F>
where
    F: Fn(&FrameContext),
{
    graphics_context: GraphicsContext<'a, F>,
}

impl<'a, F> WindowEventHandler for TestWindowEventHandler<'a, F>
where
    F: Fn(&FrameContext),
{
    fn on_window_ready(&mut self, window: Arc<Window>) {
        println!(
            "Window ready: {}x{}",
            window.inner_size().width,
            window.inner_size().height
        );

        match pollster::block_on(self.graphics_context.init(window)) {
            Ok(_) => {}
            Err(e) => show_error_popup_and_panic(e, "Failed to initialize graphics context"),
        }
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

        match self.graphics_context.redraw() {
            Ok(_) => {}
            Err(e) => show_error_popup_and_panic(e, "Failed to redraw"),
        }
    }
}

fn main() {
    let draw_frame = |frame_context: &FrameContext| {
        //println!("Draw frame: {:?}", frame_context);
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
