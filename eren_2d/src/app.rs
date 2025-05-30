use eren_core::{
    app_handler::{AppHandler, AppHandlerConfig},
    render::GraphicsLibrary,
    update::Update,
};

pub struct AppConfig {
    pub window_width: u32,
    pub window_height: u32,
    pub window_title: String,
    pub graphics_library: GraphicsLibrary,
}

pub struct App<T: Update> {
    app_handler: AppHandler<T>,
}

impl<T: Update> App<T> {
    pub fn new(config: AppConfig, root: T) -> Self {
        let app_handler = AppHandler::new(
            AppHandlerConfig {
                window_width: config.window_width,
                window_height: config.window_height,
                window_title: config.window_title,
                graphics_library: config.graphics_library,
            },
            root,
        );
        Self { app_handler }
    }

    pub fn run(&mut self) {
        self.app_handler.run();
    }
}
