use std::sync::Arc;

use eren_render_vulkan_3d::renderer::Renderer3D;
use eren_render_vulkan_core::context::GraphicsContext;
use eren_window::{
    error::show_error_popup_and_panic,
    window::{WindowConfig, WindowEventHandler, WindowLifecycleManager, WindowSize},
};
use winit::window::Window;

struct TestWindowEventHandler {
    graphics_context: GraphicsContext<Renderer3D>,
    renderer: Option<Renderer3D>,
}

impl TestWindowEventHandler {
    fn recreate_renderer(&mut self) {
        let logical_device_manager = self
            .graphics_context
            .logical_device_manager
            .as_ref()
            .unwrap();

        let swapchain_manager = self.graphics_context.swapchain_manager.as_ref().unwrap();

        let renderer = match Renderer3D::new(
            logical_device_manager.logical_device.clone(),
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

        self.graphics_context.destroy();
        self.renderer = None;
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
