use std::sync::Arc;

use eren_render_3d::renderer::Renderer3D;
use eren_render_core::context::GraphicsContext;
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

struct TestWindowEventHandler<'a> {
    graphics_context: GraphicsContext<'a, Renderer3D>,
    renderer: Option<Renderer3D>,
}

impl<'a> TestWindowEventHandler<'a> {
    fn recreate_renderer(&mut self, window_size: WindowSize) {
        let device = self.graphics_context.device.as_ref().unwrap();
        let surface_format = self.graphics_context.surface_format.unwrap();

        let renderer = Renderer3D::new(device, surface_format, window_size);

        self.renderer = Some(renderer);
    }
}

impl<'a> WindowEventHandler for TestWindowEventHandler<'a> {
    fn on_window_ready(&mut self, window: Arc<Window>) {
        match pollster::block_on(self.graphics_context.init(window.clone())) {
            Ok(_) => {}
            Err(e) => show_error_popup_and_panic(e, "Failed to initialize graphics context"),
        }

        let window_inner_size = window.inner_size();
        let window_size = WindowSize {
            width: window_inner_size.width,
            height: window_inner_size.height,
            scale_factor: window.scale_factor(),
        };

        self.recreate_renderer(window_size);
    }

    fn on_window_lost(&mut self) {
        println!("Window lost");

        self.renderer = None;
        self.graphics_context.destroy();
    }

    fn on_window_resized(&mut self, size: WindowSize) {
        println!("Window resized: {:?}", size);

        self.graphics_context.resize(size);

        if let (Some(renderer), Some(queue)) =
            (&mut self.renderer, &mut self.graphics_context.queue)
        {
            renderer.on_window_resized(queue, size);
        }
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
    match WindowLifecycleManager::new(
        WindowConfig {
            width: 800,
            height: 600,
            title: "Test Window",
            canvas_id: None,
        },
        TestWindowEventHandler {
            graphics_context: GraphicsContext::new(),
            renderer: None,
        },
    )
    .start_event_loop()
    {
        Ok(_) => {}
        Err(e) => show_error_popup_and_panic(e, "Failed to start event loop"),
    }
}
