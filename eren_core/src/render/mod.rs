#[derive(PartialEq)]
pub enum GraphicsLibrary {
    Ash,
    Wgpu,
}

pub mod ash;
pub mod gpu;
pub mod wgpu;
