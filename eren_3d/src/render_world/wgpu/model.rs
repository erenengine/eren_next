use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2],
}

#[derive(Clone)]
pub struct Model {
    pub meshes: Vec<Mesh>,
}

#[derive(Clone)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
    pub material: Material,
}

#[derive(Clone)]
pub struct Material {
    pub base_color_texture: Option<image::RgbaImage>,
    pub normal_texture: Option<image::RgbaImage>,
    pub metallic_roughness_texture: Option<image::RgbaImage>,
}
