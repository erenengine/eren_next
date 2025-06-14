use std::sync::Arc;

use eren_render_3d::renderer::Renderer3D;
use eren_render_core::context::GraphicsContext;
use eren_window::{
    error::show_error_popup_and_panic,
    window::{WindowConfig, WindowEventHandler, WindowLifecycleManager, WindowSize},
};
use winit::window::Window;

struct TestWindowEventHandler<'a> {
    graphics_context: GraphicsContext<'a, Renderer3D>,
    renderer: Option<Renderer3D>,
}

impl<'a> TestWindowEventHandler<'a> {
    fn recreate_renderer(&mut self) {
        let device = self.graphics_context.device.as_ref().unwrap();
        let surface_format = self.graphics_context.surface_format.unwrap();

        let renderer = Renderer3D::new(device, surface_format);

        self.renderer = Some(renderer);
    }
}

impl<'a> WindowEventHandler for TestWindowEventHandler<'a> {
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
                Ok(_) => {}
                Err(e) => show_error_popup_and_panic(e, "Failed to redraw"),
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
            graphics_context: GraphicsContext::new(),
            renderer: None,
        },
    )
    .start_event_loop();
}
