use glam::{Mat3, Vec2};

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
    matrix: Mat3,
    alpha: f32,
    is_dirty: bool,
}

impl GlobalTransform {
    pub fn new() -> Self {
        Self {
            matrix: Mat3::IDENTITY,
            alpha: 1.0,
            is_dirty: false,
        }
    }

    pub fn update(&mut self, parent: &GlobalTransform, local: &mut LocalTransform) {
        if parent.is_dirty || local.is_dirty {
            let pivot_transform = Mat3::from_translation(local.pivot)
                * Mat3::from_angle(local.rotation)
                * Mat3::from_scale(local.scale)
                * Mat3::from_translation(-local.pivot);

            let local_matrix =
                Mat3::from_translation(local.position - local.pivot) * pivot_transform;

            self.matrix = parent.matrix * local_matrix;
            self.alpha = parent.alpha * local.alpha;
            self.is_dirty = true;

            local.is_dirty = false;
        }
    }

    pub fn finalize(&mut self) {
        self.is_dirty = false;
    }

    pub fn extract(&mut self) -> (Mat3, f32) {
        self.finalize();
        (self.matrix, self.alpha)
    }
}
