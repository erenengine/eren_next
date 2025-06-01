use glam::{Mat2, Vec2};

pub struct LocalTransform {
    position: Vec2,
    pivot: Vec2,
    scale: Vec2,
    rotation: f32,
    alpha: f32,
    is_dirty: bool,
}

impl LocalTransform {
    pub fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            pivot: Vec2::ZERO,
            scale: Vec2::splat(1.0),
            rotation: 0.0,
            alpha: 1.0,
            is_dirty: true,
        }
    }

    pub fn position(&self) -> Vec2 {
        self.position
    }

    pub fn set_position(&mut self, position: Vec2) {
        if self.position != position {
            self.position = position;
            self.is_dirty = true;
        }
    }

    pub fn pivot(&self) -> Vec2 {
        self.pivot
    }

    pub fn set_pivot(&mut self, pivot: Vec2) {
        if self.pivot != pivot {
            self.pivot = pivot;
            self.is_dirty = true;
        }
    }

    pub fn scale(&self) -> Vec2 {
        self.scale
    }

    pub fn set_scale(&mut self, scale: Vec2) {
        if self.scale != scale {
            self.scale = scale;
            self.is_dirty = true;
        }
    }

    pub fn rotation(&self) -> f32 {
        self.rotation
    }

    pub fn set_rotation(&mut self, rotation: f32) {
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
    position: Vec2,
    scale: Vec2,
    rotation: f32,
    alpha: f32,
    is_dirty: bool,
}

impl GlobalTransform {
    pub fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            scale: Vec2::splat(1.0),
            rotation: 0.0,
            alpha: 1.0,
            is_dirty: false,
        }
    }

    pub fn position(&self) -> Vec2 {
        self.position
    }

    pub fn scale(&self) -> Vec2 {
        self.scale
    }

    pub fn rotation(&self) -> f32 {
        self.rotation
    }

    pub fn alpha(&self) -> f32 {
        self.alpha
    }

    pub fn update(&mut self, parent: &GlobalTransform, local: &LocalTransform) {
        if parent.is_dirty || local.is_dirty {
            self.scale = parent.scale * local.scale;
            self.rotation = parent.rotation + local.rotation;
            self.alpha = parent.alpha * local.alpha;

            let scaled_local_pos = local.position * parent.scale;
            let parent_rotation_matrix = Mat2::from_angle(parent.rotation);
            let rotated_scaled_local_pos = parent_rotation_matrix * scaled_local_pos;

            let scaled_pivot = local.pivot * self.scale;
            let local_rotation_matrix = Mat2::from_angle(local.rotation);
            let rotated_scaled_pivot_offset = local_rotation_matrix * scaled_pivot;

            self.position =
                parent.position + rotated_scaled_local_pos - rotated_scaled_pivot_offset;

            self.is_dirty = true;
        }
    }
}
