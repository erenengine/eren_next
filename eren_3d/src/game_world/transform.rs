use glam::{Quat, Vec3};

pub struct LocalTransform {
    position: Vec3,
    pivot: Vec3,
    scale: Vec3,
    rotation: Quat,
    alpha: f32,
    is_dirty: bool,
}

impl LocalTransform {
    pub fn new() -> Self {
        Self {
            position: Vec3::ZERO,
            pivot: Vec3::ZERO,
            scale: Vec3::ONE,
            rotation: Quat::IDENTITY,
            alpha: 1.0,
            is_dirty: true,
        }
    }
}

pub struct GlobalTransform {
    position: Vec3,
    scale: Vec3,
    rotation: Quat,
    alpha: f32,
    is_dirty: bool,
}

impl GlobalTransform {
    pub fn new() -> Self {
        Self {
            position: Vec3::ZERO,
            scale: Vec3::ONE,
            rotation: Quat::IDENTITY,
            alpha: 1.0,
            is_dirty: false,
        }
    }
}
