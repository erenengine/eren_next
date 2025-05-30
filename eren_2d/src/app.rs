use eren_core::render::GraphicsLibrary;

pub struct AppConfig {
    graphics_library: GraphicsLibrary,
}

pub struct App {
    config: AppConfig,
}

impl App {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }
}
