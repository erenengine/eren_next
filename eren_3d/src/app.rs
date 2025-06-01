use std::hash::Hash;

use eren_core::{
    render_world::{
        ash::gpu::AshGpuResourceManager,
        common::gpu::{GpuResourceManager, GraphicsLibrary},
        wgpu::gpu::WgpuGpuResourceManager,
    },
    window::{WindowConfig, WindowLifecycleManager},
};

use crate::{
    game_world::nodes::game_node::GameNode,
    render_world::{ash::engine::AshEngine3D, wgpu::engine::WgpuEngine3D},
};

pub struct AppConfig {
    pub window_width: u32,
    pub window_height: u32,
    pub window_title: String,
    pub graphics_library: GraphicsLibrary,
}

pub struct App {
    window_lifecycle_manager: WindowLifecycleManager,
}

impl App {
    pub fn new<R: GameNode<SA> + 'static, SA: Eq + Hash + Ord + Copy + 'static>(
        config: AppConfig,
        root_node: R,
    ) -> Self {
        let gpu_resource_manager: Box<dyn GpuResourceManager>;
        if config.graphics_library == GraphicsLibrary::Ash {
            let engine = AshEngine3D::new(root_node);
            gpu_resource_manager = Box::new(AshGpuResourceManager::new(Box::new(engine)));
        } else if config.graphics_library == GraphicsLibrary::Wgpu {
            let engine = WgpuEngine3D::new(root_node);
            gpu_resource_manager = Box::new(WgpuGpuResourceManager::new(Box::new(engine)));
        } else {
            panic!("Invalid graphics library");
        }
        let window_lifecycle_manager = WindowLifecycleManager::new(
            WindowConfig {
                window_width: config.window_width,
                window_height: config.window_height,
                window_title: config.window_title,
            },
            gpu_resource_manager,
        );
        Self {
            window_lifecycle_manager,
        }
    }

    pub fn run(&mut self) {
        self.window_lifecycle_manager.run();
    }
}
