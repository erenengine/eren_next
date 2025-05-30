#[derive(PartialEq)]
pub enum GraphicsLibrary {
    Wgpu,
    Ash,
}

pub mod ash;
pub mod gpu;
pub mod wgpu;
