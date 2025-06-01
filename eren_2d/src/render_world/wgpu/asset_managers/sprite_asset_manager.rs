use std::{collections::HashMap, hash::Hash};

use glam::Vec2;

pub struct SpriteGpuResource {
    pub size: Vec2,
    pub bind_group: wgpu::BindGroup,
}

pub struct WgpuSpriteAssetManager<SA> {
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    bind_group_layout: Option<wgpu::BindGroupLayout>,
    sampler: Option<wgpu::Sampler>,

    loading_assets: Vec<SA>,
    loaded_images: HashMap<SA, image::RgbaImage>,

    gpu_resources: HashMap<SA, SpriteGpuResource>,
}

impl<SA: Eq + Hash + Clone> WgpuSpriteAssetManager<SA> {
    pub fn new() -> Self {
        Self {
            device: None,
            queue: None,
            bind_group_layout: None,
            sampler: None,

            loading_assets: Vec::new(),
            loaded_images: HashMap::new(),

            gpu_resources: HashMap::new(),
        }
    }

    fn create_bind_group(&mut self, asset: SA, image: &image::RgbaImage) {
        if let (Some(device), Some(queue), Some(bind_group_layout), Some(sampler)) = (
            &self.device,
            &self.queue,
            &self.bind_group_layout,
            &self.sampler,
        ) {
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

            self.gpu_resources.insert(
                asset,
                SpriteGpuResource {
                    size: Vec2::new(width as f32, height as f32),
                    bind_group: device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("sprite bind group"),
                        layout: bind_group_layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureView(
                                    &texture.create_view(&Default::default()),
                                ),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::Sampler(sampler),
                            },
                        ],
                    }),
                },
            );
        }
    }

    pub fn on_gpu_resources_ready(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) {
        self.device = Some(device.clone());
        self.queue = Some(queue.clone());
        self.bind_group_layout = Some(bind_group_layout.clone());

        self.sampler = Some(device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("sprite sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        }));

        let images: Vec<(SA, image::RgbaImage)> = self
            .loaded_images
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        for (asset, image) in images {
            self.create_bind_group(asset, &image);
        }
    }

    pub fn on_gpu_resources_lost(&mut self) {
        self.device = None;
        self.queue = None;
        self.bind_group_layout = None;
        self.sampler = None;
        self.gpu_resources.clear();
    }

    pub fn load_sprite(&mut self, asset: SA, path: String) {
        let image = image::open(path.clone()).unwrap().to_rgba8();
        self.loaded_images.insert(asset.clone(), image.clone());
        self.create_bind_group(asset, &image);
    }

    pub fn get_gpu_resource(&self, asset: SA) -> Option<&SpriteGpuResource> {
        self.gpu_resources.get(&asset)
    }
}
