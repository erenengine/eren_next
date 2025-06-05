use glam::{Mat4, Quat, Vec3};

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

    pub fn set_position(&mut self, value: Vec3) {
        if self.position != value {
            self.position = value;
            self.is_dirty = true;
        }
    }

    pub fn pivot(&self) -> Vec3 {
        self.pivot
    }

    pub fn set_pivot(&mut self, value: Vec3) {
        if self.pivot != value {
            self.pivot = value;
            self.is_dirty = true;
        }
    }

    pub fn scale(&self) -> Vec3 {
        self.scale
    }

    pub fn set_scale(&mut self, value: Vec3) {
        if self.scale != value {
            self.scale = value;
            self.is_dirty = true;
        }
    }

    pub fn rotation(&self) -> Quat {
        self.rotation
    }

    pub fn set_rotation(&mut self, value: Quat) {
        if self.rotation != value {
            self.rotation = value;
            self.is_dirty = true;
        }
    }

    pub fn alpha(&self) -> f32 {
        self.alpha
    }

    pub fn set_alpha(&mut self, value: f32) {
        if self.alpha != value {
            self.alpha = value;
            self.is_dirty = true;
        }
    }
}

pub struct GlobalTransform {
    matrix: Mat4,
    alpha: f32,
    is_dirty: bool,
}

impl GlobalTransform {
    pub fn new() -> Self {
        Self {
            matrix: Mat4::IDENTITY,
            alpha: 1.0,
            is_dirty: false,
        }
    }

    pub fn update(&mut self, parent: &GlobalTransform, local: &mut LocalTransform) {
        if parent.is_dirty || local.is_dirty {
            let pivot_transform = Mat4::from_translation(local.pivot)
                * Mat4::from_quat(local.rotation)
                * Mat4::from_scale(local.scale)
                * Mat4::from_translation(-local.pivot);

            let local_matrix =
                Mat4::from_translation(local.position - local.pivot) * pivot_transform;

            self.matrix = parent.matrix * local_matrix;
            self.alpha = parent.alpha * local.alpha;
            self.is_dirty = true;

            local.is_dirty = false;
        }
    }

    pub fn finalize(&mut self) {
        self.is_dirty = false;
    }

    pub fn extract(&mut self) -> (Mat4, f32) {
        self.finalize();
        (self.matrix, self.alpha)
    }
}
