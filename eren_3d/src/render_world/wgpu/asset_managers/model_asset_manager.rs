use std::{collections::HashMap, hash::Hash};

use crate::render_world::wgpu::{load_model::load_gltf_model::load_gltf_model, model::Model};
use wgpu::util::DeviceExt;

pub struct MeshGpuResource {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub bind_group: Option<wgpu::BindGroup>,
}

pub struct ModelGpuResource {
    pub meshes: Vec<MeshGpuResource>,
}

pub struct WgpuModelAssetManager<MA> {
    device: Option<wgpu::Device>,
    queue: Option<wgpu::Queue>,
    sampler: Option<wgpu::Sampler>,
    bind_group_layout: Option<wgpu::BindGroupLayout>,

    loading_assets: Vec<MA>,
    loaded_models: HashMap<MA, Model>,

    gpu_resources: HashMap<MA, ModelGpuResource>,
}

impl<MA: Eq + Hash + Clone> WgpuModelAssetManager<MA> {
    pub fn new() -> Self {
        Self {
            device: None,
            queue: None,
            sampler: None,
            bind_group_layout: None,

            loading_assets: Vec::new(),
            loaded_models: HashMap::new(),

            gpu_resources: HashMap::new(),
        }
    }

    fn create_gpu_resource(&mut self, asset: MA, model: &Model) {
        if let (Some(device), Some(queue), Some(bind_group_layout), Some(sampler)) = (
            &self.device,
            &self.queue,
            &self.bind_group_layout,
            &self.sampler,
        ) {
            let mut mesh_resources = Vec::new();

            for mesh in &model.meshes {
                let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("mesh_vertex_buffer"),
                    contents: bytemuck::cast_slice(&mesh.vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });

                let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("mesh_index_buffer"),
                    contents: bytemuck::cast_slice(&mesh.indices),
                    usage: wgpu::BufferUsages::INDEX,
                });

                let bind_group = if let Some(texture_image) = &mesh.material.base_color_texture {
                    let (width, height) = texture_image.dimensions();
                    let size = wgpu::Extent3d {
                        width,
                        height,
                        depth_or_array_layers: 1,
                    };

                    let texture = device.create_texture(&wgpu::TextureDescriptor {
                        label: Some("mesh_texture"),
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
                        texture_image,
                        wgpu::TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(4 * width),
                            rows_per_image: Some(height),
                        },
                        size,
                    );

                    Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("mesh_bind_group"),
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
                    }))
                } else {
                    None
                };

                mesh_resources.push(MeshGpuResource {
                    vertex_buffer,
                    index_buffer,
                    num_indices: mesh.indices.len() as u32,
                    bind_group,
                });
            }

            self.gpu_resources.insert(
                asset,
                ModelGpuResource {
                    meshes: mesh_resources,
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

        let models: Vec<(MA, Model)> = self
            .loaded_models
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        for (asset, model) in models {
            self.create_gpu_resource(asset, &model);
        }
    }

    pub fn on_gpu_resources_lost(&mut self) {
        self.device = None;
        self.queue = None;
        self.bind_group_layout = None;
        self.sampler = None;
        self.gpu_resources.clear();
    }

    pub fn load_model(&mut self, asset: MA, path: String) {
        let extension = path.split('.').last().unwrap();
        if extension == "gltf" || extension == "glb" {
            let model = load_gltf_model(path);
            self.loaded_models.insert(asset.clone(), model.clone());
            self.create_gpu_resource(asset, &model);
        } else {
            println!("Unsupported model format: {}", extension);
        }
    }

    pub fn get_gpu_resource(&self, asset: MA) -> Option<&ModelGpuResource> {
        self.gpu_resources.get(&asset)
    }
}
