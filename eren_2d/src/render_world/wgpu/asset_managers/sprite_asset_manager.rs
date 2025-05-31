use std::{collections::HashMap, hash::Hash};

pub struct WgpuSpriteAssetManager<SA> {
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,

    loading_assets: Vec<SA>,
    loaded_images: HashMap<SA, image::RgbaImage>,
    texture_views: HashMap<SA, wgpu::TextureView>,
}

impl<SA: Eq + Hash + Clone> WgpuSpriteAssetManager<SA> {
    pub fn new() -> Self {
        Self {
            device: None,
            queue: None,

            loading_assets: Vec::new(),
            loaded_images: HashMap::new(),
            texture_views: HashMap::new(),
        }
    }

    fn upload_texture(&mut self, asset: SA, image: &image::RgbaImage) {
        if let (Some(device), Some(queue)) = (&self.device, &self.queue) {
            let (width, height) = image.dimensions();
            let size = wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            };

            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: None,
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });

            self.texture_views
                .insert(asset, texture.create_view(&Default::default()));

            queue.write_texture(
                texture.as_image_copy(),
                image,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(width * 4),
                    rows_per_image: Some(height),
                },
                size,
            );
        }
    }

    pub fn on_gpu_resources_ready(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.device = Some(device.clone());
        self.queue = Some(queue.clone());

        let images: Vec<(SA, image::RgbaImage)> = self
            .loaded_images
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        for (asset, image) in images {
            self.upload_texture(asset, &image);
        }
    }

    pub fn on_gpu_resources_lost(&mut self) {
        self.device = None;
        self.queue = None;
        self.texture_views.clear();
    }

    pub fn load_sprite(&mut self, asset: SA, path: String) {
        let image = image::open(path.clone()).unwrap().to_rgba8();
        self.loaded_images.insert(asset.clone(), image.clone());
        self.upload_texture(asset, &image);
    }

    pub fn get_texture_view(&self, asset: SA) -> Option<&wgpu::TextureView> {
        self.texture_views.get(&asset)
    }
}
