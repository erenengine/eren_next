use std::hash::Hash;

#[cfg(not(target_arch = "wasm32"))]
use eren_core::render_world::ash::gpu::AshGpuResourceManager;

#[cfg(not(target_arch = "wasm32"))]
use crate::render_world::ash::engine::AshEngine2D;

use eren_core::{
    render_world::{
        common::gpu::{GpuResourceManager, GraphicsLibrary},
        wgpu::gpu::WgpuGpuResourceManager,
    },
    window::{WindowConfig, WindowLifecycleManager},
};

use crate::{game_world::nodes::game_node::GameNode, render_world::wgpu::engine::WgpuEngine2D};

pub struct AppConfig {
    pub window_width: u32,
    pub window_height: u32,
    pub window_title: String,
    pub graphics_library: GraphicsLibrary,
}

pub struct App {
    window_lifecycle_manager: WindowLifecycleManager,
}

#[cfg(not(target_arch = "wasm32"))]
fn make_gpu_manager<R: GameNode<SA>, SA>(
    lib: GraphicsLibrary,
    root_node: R,
) -> Box<dyn GpuResourceManager>
where
    SA: Eq + Hash + Ord + Copy + 'static,
    R: 'static,
{
    match lib {
        GraphicsLibrary::Ash => {
            let engine = AshEngine2D::new(root_node);
            Box::new(AshGpuResourceManager::new(Box::new(engine)))
        }
        GraphicsLibrary::Wgpu => {
            let engine = WgpuEngine2D::new(root_node);
            Box::new(WgpuGpuResourceManager::new(Box::new(engine)))
        }
        _ => panic!("Invalid graphics library"),
    }
}

#[cfg(target_arch = "wasm32")]
fn make_gpu_manager<R: GameNode<SA>, SA>(
    lib: GraphicsLibrary,
    root_node: R,
) -> Box<dyn GpuResourceManager>
where
    SA: Eq + Hash + Ord + Copy + 'static,
    R: 'static,
{
    match lib {
        GraphicsLibrary::Wgpu => {
            let engine = WgpuEngine2D::new(root_node);
            Box::new(WgpuGpuResourceManager::new(Box::new(engine)))
        }
        _ => panic!("Ash is not supported on wasm32"),
    }
}

impl App {
    pub fn new<R: GameNode<SA> + 'static, SA: Eq + Hash + Ord + Copy + 'static>(
        config: AppConfig,
        root_node: R,
    ) -> Self {
        let window_lifecycle_manager = WindowLifecycleManager::new(
            WindowConfig {
                window_width: config.window_width,
                window_height: config.window_height,
                window_title: config.window_title,
            },
            make_gpu_manager(config.graphics_library, root_node),
        );
        Self {
            window_lifecycle_manager,
        }
    }

    pub fn run(&mut self) {
        self.window_lifecycle_manager.run();
    }
}
