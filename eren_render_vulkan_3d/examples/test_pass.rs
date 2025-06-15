use std::sync::Arc;

use eren_render_vulkan_3d::render::test_renderer::TestRenderer;
use eren_render_vulkan_core::context::GraphicsContext;
use eren_window::window::{WindowConfig, WindowEventHandler, WindowLifecycleManager, WindowSize};
use winit::window::Window;

use native_dialog::{DialogBuilder, MessageLevel};

pub fn show_error_popup_and_panic<E: std::fmt::Display>(error: E, context: &str) -> ! {
    DialogBuilder::message()
        .set_level(MessageLevel::Error)
        .set_title(context)
        .set_text(error.to_string())
        .alert()
        .show()
        .unwrap();
    panic!("{}: {}", context, error);
}

struct TestWindowEventHandler {
    graphics_context: GraphicsContext,
    renderer: Option<TestRenderer>,
}

impl TestWindowEventHandler {
    fn recreate_renderer(&mut self) {
        let device_manager = self.graphics_context.device_manager.as_ref().unwrap();
        let swapchain_manager = self.graphics_context.swapchain_manager.as_ref().unwrap();

        let renderer = match TestRenderer::new(
            device_manager.device.clone(),
            &self.graphics_context.swapchain_image_views,
            swapchain_manager.preferred_surface_format,
            swapchain_manager.image_extent,
        ) {
            Ok(renderer) => renderer,
            Err(e) => show_error_popup_and_panic(e, "Failed to create renderer"),
        };

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
            match self.graphics_context.redraw(renderer, &[]) {
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
    match WindowLifecycleManager::new(
        WindowConfig {
            width: 800,
            height: 600,
            title: "Test Window",
            canvas_id: None,
        },
        TestWindowEventHandler {
            graphics_context: match GraphicsContext::new() {
                Ok(graphics_context) => graphics_context,
                Err(e) => show_error_popup_and_panic(e, "Failed to create graphics context"),
            },
            renderer: None,
        },
    )
    .start_event_loop()
    {
        Ok(_) => {}
        Err(e) => show_error_popup_and_panic(e, "Failed to start event loop"),
    }
}
