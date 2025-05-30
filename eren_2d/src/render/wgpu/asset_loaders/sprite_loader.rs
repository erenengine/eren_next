use std::collections::HashMap;

use eren_core::render::wgpu::asset::WgpuAssetLoader;

pub struct WgpuSpriteLoader {
    images: HashMap<String, image::RgbaImage>,
    texture_views: HashMap<String, wgpu::TextureView>,
}

impl WgpuSpriteLoader {
    pub fn new() -> Self {
        Self {
            images: HashMap::new(),
            texture_views: HashMap::new(),
        }
    }
}

impl WgpuAssetLoader for WgpuSpriteLoader {
    fn load(&mut self, path: String) {
        self.images
            .insert(path.clone(), image::open(path).unwrap().to_rgba8());
    }

    fn surface_created(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        for (path, img) in &self.images {
            let (width, height) = img.dimensions();
            let size = wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            };

            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some(path),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });

            self.texture_views
                .insert(path.to_string(), texture.create_view(&Default::default()));

            queue.write_texture(
                texture.as_image_copy(),
                img,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(width * 4),
                    rows_per_image: Some(height),
                },
                size,
            );
        }
    }

    fn surface_destroyed(&mut self) {
        self.texture_views.clear();
    }
}
