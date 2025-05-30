use eren_core::{
    app_handler::{AppHandler, AppHandlerConfig},
    render::{
        GraphicsLibrary, ash::gpu::AshGpuContext, gpu::GpuContext, wgpu::gpu::WgpuGpuContext,
    },
    update::Update,
};

use crate::render::wgpu::{
    asset_loaders::sprite_loader::WgpuSpriteLoader,
    passes::sprite_render_pass::WgpuSpriteRenderPass,
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
        let gpu_context: Box<dyn GpuContext>;

        if config.graphics_library == GraphicsLibrary::Ash {
            gpu_context = Box::new(AshGpuContext::new());
        } else {
            let mut wgpu_gpu_context = WgpuGpuContext::new();
            wgpu_gpu_context
                .asset_manager
                .add_loader("png".into(), Box::new(WgpuSpriteLoader::new()));
            wgpu_gpu_context
                .asset_manager
                .add_loader("jpg".into(), Box::new(WgpuSpriteLoader::new()));
            wgpu_gpu_context
                .asset_manager
                .add_loader("jpeg".into(), Box::new(WgpuSpriteLoader::new()));
            wgpu_gpu_context.add_render_pass(Box::new(WgpuSpriteRenderPass::new()));
            gpu_context = Box::new(wgpu_gpu_context);
        }

        let app_handler = AppHandler::new(
            AppHandlerConfig {
                window_width: config.window_width,
                window_height: config.window_height,
                window_title: config.window_title,
            },
            gpu_context,
            root,
        );
        Self { app_handler }
    }

    pub fn run(&mut self) {
        self.app_handler.run();
    }
}
