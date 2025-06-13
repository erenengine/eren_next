use std::sync::Arc;

use eren_render_vulkan_core::{
    context::GraphicsContext,
    renderer::{FrameContext, Renderer},
};
use eren_window::{
    error::show_error_popup_and_panic,
    window::{WindowConfig, WindowEventHandler, WindowLifecycleManager, WindowSize},
};
use winit::window::Window;

struct EmptyRenderer;

impl Renderer for EmptyRenderer {
    fn render(&self, _frame_context: &FrameContext) {}
}

struct TestWindowEventHandler {
    graphics_context: GraphicsContext<EmptyRenderer>,
    renderer: Option<EmptyRenderer>,
}

impl TestWindowEventHandler {
    fn recreate_renderer(&mut self) {
        let renderer = EmptyRenderer;

        self.renderer = Some(renderer);
    }
}

impl WindowEventHandler for TestWindowEventHandler {
    fn on_window_ready(&mut self, window: Arc<Window>) {
        println!(
            "Window ready: {}x{}",
            window.inner_size().width,
            window.inner_size().height
        );

        match self.graphics_context.init(window) {
            Ok(_) => {}
            Err(e) => show_error_popup_and_panic(e, "Failed to initialize graphics context"),
        };

        self.recreate_renderer();
    }

    fn on_window_lost(&mut self) {
        println!("Window lost");

        self.renderer = None;
        self.graphics_context.destroy();
    }

    fn on_window_resized(&mut self, size: WindowSize) {
        println!("Window resized: {:?}", size);

        self.graphics_context.resize(size);
    }

    fn redraw(&mut self) {
        if let Some(renderer) = &self.renderer {
            match self.graphics_context.redraw(renderer) {
                Ok(renderer_needs_recreation) => {
                    if renderer_needs_recreation {
                        self.recreate_renderer();
                    }
                }
                Err(e) => show_error_popup_and_panic(e, "Failed to redraw graphics context"),
            }
        }
    }

    fn on_window_close_requested(&mut self) {
        self.renderer = None;
        self.graphics_context.destroy();
    }
}

fn main() {
    WindowLifecycleManager::new(
        WindowConfig {
            width: 800,
            height: 600,
            title: "Test Window",
        },
        TestWindowEventHandler {
            graphics_context: match GraphicsContext::new() {
                Ok(graphics_context) => graphics_context,
                Err(e) => show_error_popup_and_panic(e, "Failed to create graphics context"),
            },
            renderer: None,
        },
    )
    .start_event_loop();
}
