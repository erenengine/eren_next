use std::collections::HashMap;

use eren_core::render_world::wgpu::asset::WgpuAssetLoader;

pub struct WgpuSpriteLoader {
    images: HashMap<String, image::RgbaImage>,
    texture_views: HashMap<String, wgpu::TextureView>,

    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
}

impl WgpuSpriteLoader {
    pub fn new() -> Self {
        Self {
            images: HashMap::new(),
            texture_views: HashMap::new(),

            device: None,
            queue: None,
        }
    }

    fn upload_texture(&mut self, path: String, image: &image::RgbaImage) {
        if let (Some(device), Some(queue)) = (&self.device, &self.queue) {
            let (width, height) = image.dimensions();
            let size = wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            };

            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some(&path),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });

            self.texture_views
                .insert(path, texture.create_view(&Default::default()));

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
}

impl WgpuAssetLoader for WgpuSpriteLoader {
    fn load(&mut self, path: String) {
        let image = image::open(path.clone()).unwrap().to_rgba8();
        self.images.insert(path.clone(), image.clone());
        self.upload_texture(path.clone(), &image);
    }

    fn upload_textures(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        self.device = Some(device.clone());
        self.queue = Some(queue.clone());

        let images: Vec<(String, image::RgbaImage)> = self
            .images
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        for (path, image) in images {
            self.upload_texture(path, &image);
        }
    }

    fn unload_textures(&mut self) {
        self.texture_views.clear();
    }
}
