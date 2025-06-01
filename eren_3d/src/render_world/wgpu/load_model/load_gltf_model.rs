use crate::render_world::wgpu::model::{Material, Mesh, Model, Vertex};

use gltf;
use image::{self, RgbaImage};

fn load_texture_image(
    texture: &gltf::texture::Texture,
    images: &[gltf::image::Data],
) -> Option<RgbaImage> {
    let img = &images[texture.source().index()];

    match img.format {
        gltf::image::Format::R8G8B8A8 => {
            RgbaImage::from_raw(img.width, img.height, img.pixels.clone())
        }
        gltf::image::Format::R8G8B8 => {
            let mut rgba = Vec::with_capacity((img.width * img.height * 4) as usize);
            for chunk in img.pixels.chunks_exact(3) {
                rgba.extend_from_slice(&[chunk[0], chunk[1], chunk[2], 255]);
            }
            RgbaImage::from_raw(img.width, img.height, rgba)
        }
        _ => None,
    }
}

pub fn load_gltf_model(path: String) -> Model {
    let (document, buffers, images) = gltf::import(path).expect("Failed to load GLTF");

    let mut meshes = Vec::new();

    for mesh in document.meshes() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            let positions: Vec<[f32; 3]> = reader
                .read_positions()
                .expect("Mesh primitive is missing positions")
                .collect();

            let normals: Vec<[f32; 3]> = reader
                .read_normals()
                .map(|n| n.collect())
                .unwrap_or_else(|| vec![[0.0, 0.0, 1.0]; positions.len()]);

            let texcoords: Vec<[f32; 2]> = reader
                .read_tex_coords(0)
                .map(|c| c.into_f32().collect())
                .unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);

            let vertices: Vec<Vertex> = positions
                .iter()
                .zip(normals.iter())
                .zip(texcoords.iter())
                .map(|((p, n), t)| Vertex {
                    position: *p,
                    normal: *n,
                    tex_coords: *t,
                })
                .collect();

            let indices: Vec<u32> = reader
                .read_indices()
                .map(|r| r.into_u32().collect())
                .unwrap_or_default();

            let pbr = primitive.material().pbr_metallic_roughness();

            let base_color_texture = pbr
                .base_color_texture()
                .and_then(|t| load_texture_image(&t.texture(), &images));

            let normal_texture = primitive
                .material()
                .normal_texture()
                .and_then(|t| load_texture_image(&t.texture(), &images));

            let metallic_roughness_texture = pbr
                .metallic_roughness_texture()
                .and_then(|t| load_texture_image(&t.texture(), &images));

            meshes.push(Mesh {
                vertices,
                indices,
                material: Material {
                    base_color_texture,
                    normal_texture,
                    metallic_roughness_texture,
                },
            });
        }
    }

    Model { meshes }
}
