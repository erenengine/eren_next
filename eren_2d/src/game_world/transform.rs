use glam::Vec2;

pub struct LocalTransform {
    position: Vec2,
    pivot: Vec2,
    scale: Vec2,
    rotation: f32,
    alpha: f32,
    is_dirty: bool,
}

impl LocalTransform {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            position: Vec2::new(x, y),
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

    pub fn set_position(&mut self, position: Vec2) {
        if self.position != position {
            self.position = position;
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

    pub fn alpha(&self) -> f32 {
        self.alpha
    }

    pub fn update(&mut self, parent: &GlobalTransform, local: &LocalTransform) {
        if parent.is_dirty || local.is_dirty {
            let rx = local.position.x * parent.scale.x;
            let ry = local.position.y * parent.scale.y;

            let p_cos = parent.rotation.cos();
            let p_sin = parent.rotation.sin();

            self.scale.x = parent.scale.x * local.scale.x;
            self.scale.y = parent.scale.y * local.scale.y;

            let pivot_x = local.pivot.x * self.scale.x;
            let pivot_y = local.pivot.y * self.scale.y;

            let cos = local.rotation.cos();
            let sin = local.rotation.sin();

            self.position.x =
                parent.position.x + (rx * p_cos - ry * p_sin) - (pivot_x * cos - pivot_y * sin);
            self.position.y =
                parent.position.y + (rx * p_sin + ry * p_cos) - (pivot_x * sin + pivot_y * cos);

            self.rotation = parent.rotation + local.rotation;
            self.alpha = parent.alpha * local.alpha;

            self.is_dirty = true;
        }
    }
}
