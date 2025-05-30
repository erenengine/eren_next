use eren_core::render::RenderPassHandler;

pub struct App {
    render_passes: Vec<Box<dyn RenderPassHandler>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            render_passes: Vec::new(),
        }
    }
}
