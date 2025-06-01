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

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn set_position(&mut self, position: Vec3) {
        if self.position != position {
            self.position = position;
            self.is_dirty = true;
        }
    }

    pub fn pivot(&self) -> Vec3 {
        self.pivot
    }

    pub fn set_pivot(&mut self, pivot: Vec3) {
        if self.pivot != pivot {
            self.pivot = pivot;
            self.is_dirty = true;
        }
    }

    pub fn scale(&self) -> Vec3 {
        self.scale
    }

    pub fn set_scale(&mut self, scale: Vec3) {
        if self.scale != scale {
            self.scale = scale;
            self.is_dirty = true;
        }
    }

    pub fn rotation(&self) -> Quat {
        self.rotation
    }

    pub fn set_rotation(&mut self, rotation: Quat) {
        if self.rotation != rotation {
            self.rotation = rotation;
            self.is_dirty = true;
        }
    }

    pub fn alpha(&self) -> f32 {
        self.alpha
    }

    pub fn set_alpha(&mut self, alpha: f32) {
        if self.alpha != alpha {
            self.alpha = alpha;
            self.is_dirty = true;
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

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn scale(&self) -> Vec3 {
        self.scale
    }

    pub fn rotation(&self) -> Quat {
        self.rotation
    }

    pub fn alpha(&self) -> f32 {
        self.alpha
    }

    pub fn update(&mut self, parent: &GlobalTransform, local: &LocalTransform) {
        if parent.is_dirty || local.is_dirty {
            let rx = local.position.x * parent.scale.x;
            let ry = local.position.y * parent.scale.y;
            let rz = local.position.z * parent.scale.z;

            let p_cos = parent.rotation.x;
            let p_sin = parent.rotation.y;

            self.scale.x = parent.scale.x * local.scale.x;
            self.scale.y = parent.scale.y * local.scale.y;
            self.scale.z = parent.scale.z * local.scale.z;

            let pivot_x = local.pivot.x * self.scale.x;
            let pivot_y = local.pivot.y * self.scale.y;
            let pivot_z = local.pivot.z * self.scale.z;

            let cos = local.rotation.x;
            let sin = local.rotation.y;

            self.position.x =
                parent.position.x + (rx * p_cos - ry * p_sin) - (pivot_x * cos - pivot_y * sin);
            self.position.y =
                parent.position.y + (rx * p_sin + ry * p_cos) - (pivot_x * sin + pivot_y * cos);
            self.position.z = parent.position.z + (rz * p_cos) - (pivot_z * cos);

            self.rotation = parent.rotation * local.rotation;
            self.alpha = parent.alpha * local.alpha;

            self.is_dirty = true;
        }
    }
}
