use crate::render_world::wgpu::model::{Material, Mesh, Model, Vertex};

fn load_texture_image(
    texture: &gltf::texture::Texture,
    buffers: &[gltf::buffer::Data],
) -> Option<image::RgbaImage> {
    match texture.source().source() {
        gltf::image::Source::View { view, .. } => {
            let buffer = &buffers[view.buffer().index()];
            let start = view.offset();
            let end = start + view.length();
            image::load_from_memory(&buffer[start..end])
                .ok()
                .map(|img| img.to_rgba8())
        }
        _ => None,
    }
}

pub fn load_gltf_model(path: String) -> Model {
    let (document, buffers, _) = gltf::import(path).expect("Failed to load GLTF");

    let mut meshes = Vec::new();

    for mesh in document.meshes() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            let positions = reader.read_positions().unwrap().collect::<Vec<_>>();

            let normals: Vec<[f32; 3]> = if let Some(normals_iter) = reader.read_normals() {
                normals_iter.collect()
            } else {
                vec![[0.0, 0.0, 1.0]; positions.len()]
            };

            let texcoords = reader
                .read_tex_coords(0)
                .map(|c| c.into_f32().collect())
                .unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);

            let mut vertices = Vec::with_capacity(positions.len());
            for i in 0..positions.len() {
                vertices.push(Vertex {
                    position: positions[i],
                    normal: normals[i],
                    tex_coords: texcoords[i],
                });
            }

            let indices = reader
                .read_indices()
                .map(|r| r.into_u32().collect::<Vec<_>>())
                .unwrap_or_default();

            let mat = primitive.material().pbr_metallic_roughness();

            // base color texture
            let base_color_texture = mat
                .base_color_texture()
                .and_then(|t| load_texture_image(&t.texture(), &buffers));

            let normal_texture = primitive
                .material()
                .normal_texture()
                .and_then(|t| load_texture_image(&t.texture(), &buffers));

            let metallic_roughness_texture = mat
                .metallic_roughness_texture()
                .and_then(|t| load_texture_image(&t.texture(), &buffers));

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
