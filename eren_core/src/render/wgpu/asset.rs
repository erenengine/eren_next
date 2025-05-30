use std::collections::HashMap;

use crate::{asset::AssetManager, game::state::GameState};

pub trait WgpuAssetLoader {
    fn load(&mut self, path: String);
    fn surface_created(&mut self, device: &wgpu::Device, queue: &wgpu::Queue);
    fn surface_destroyed(&mut self);
}

pub struct WgpuAssetManager {
    asset_loaders: HashMap<String, Box<dyn WgpuAssetLoader>>,
    loading_assets: Vec<String>,
}

impl WgpuAssetManager {
    pub fn new() -> Self {
        Self {
            asset_loaders: HashMap::new(),
            loading_assets: Vec::new(),
        }
    }

    pub fn add_loader(&mut self, extension: String, loader: Box<dyn WgpuAssetLoader>) {
        self.asset_loaders.insert(extension, loader);
    }

    pub fn surface_created(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        for (_, loader) in &mut self.asset_loaders {
            loader.surface_created(device, queue);
        }
    }

    pub fn surface_destroyed(&mut self) {
        for (_, loader) in &mut self.asset_loaders {
            loader.surface_destroyed();
        }
    }
}

impl AssetManager for WgpuAssetManager {
    fn ensure_asset_loaded(&mut self, state: &mut GameState) {
        for asset in state.required_assets.iter() {
            if !self.loading_assets.contains(asset) {
                self.loading_assets.push(asset.clone());

                let extension = asset.split('.').last().unwrap().to_string();
                if let Some(loader) = self.asset_loaders.get_mut(&extension) {
                    loader.load(asset.clone());
                }
            }
        }
    }
}
