use eren_2d::app::{App, AppConfig};
use eren_core::{game::context::GameContext, render::GraphicsLibrary, update::Update};

struct Root {}

impl Root {
    pub fn new() -> Self {
        Self {}
    }
}

impl Update for Root {
    fn update(&mut self, state: &mut GameContext) {}
}

fn main() {
    App::new(
        AppConfig {
            window_width: 800,
            window_height: 600,
            window_title: "Test Sprite".to_string(),
            graphics_library: GraphicsLibrary::Wgpu,
        },
        Root::new(),
    )
    .run();
}
