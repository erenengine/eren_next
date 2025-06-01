pub struct LocalTransform {
    x: f32,
    y: f32,
    pivot_x: f32,
    pivot_y: f32,
    scale_x: f32,
    scale_y: f32,
    rotation: f32,
    alpha: f32,
    is_dirty: bool,
}

impl LocalTransform {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            pivot_x: 0.0,
            pivot_y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotation: 0.0,
            alpha: 1.0,
            is_dirty: true,
        }
    }

    pub fn x(&self) -> f32 {
        self.x
    }

    pub fn y(&self) -> f32 {
        self.y
    }

    pub fn set_x(&mut self, x: f32) {
        if self.x != x {
            self.x = x;
            self.is_dirty = true;
        }
    }

    pub fn set_y(&mut self, y: f32) {
        if self.y != y {
            self.y = y;
            self.is_dirty = true;
        }
    }

    pub fn set_position(&mut self, x: f32, y: f32) {
        if self.x != x || self.y != y {
            self.x = x;
            self.y = y;
            self.is_dirty = true;
        }
    }

    pub fn pivot_x(&self) -> f32 {
        self.pivot_x
    }

    pub fn pivot_y(&self) -> f32 {
        self.pivot_y
    }

    pub fn set_pivot_x(&mut self, pivot_x: f32) {
        if self.pivot_x != pivot_x {
            self.pivot_x = pivot_x;
            self.is_dirty = true;
        }
    }

    pub fn set_pivot_y(&mut self, pivot_y: f32) {
        if self.pivot_y != pivot_y {
            self.pivot_y = pivot_y;
            self.is_dirty = true;
        }
    }

    pub fn set_pivot(&mut self, pivot_x: f32, pivot_y: f32) {
        if self.pivot_x != pivot_x || self.pivot_y != pivot_y {
            self.pivot_x = pivot_x;
            self.pivot_y = pivot_y;
            self.is_dirty = true;
        }
    }

    pub fn scale_x(&self) -> f32 {
        self.scale_x
    }

    pub fn scale_y(&self) -> f32 {
        self.scale_y
    }

    pub fn set_scale_x(&mut self, scale_x: f32) {
        if self.scale_x != scale_x {
            self.scale_x = scale_x;
            self.is_dirty = true;
        }
    }

    pub fn set_scale_y(&mut self, scale_y: f32) {
        if self.scale_y != scale_y {
            self.scale_y = scale_y;
            self.is_dirty = true;
        }
    }

    pub fn set_scale(&mut self, scale_x: f32, scale_y: f32) {
        if self.scale_x != scale_x || self.scale_y != scale_y {
            self.scale_x = scale_x;
            self.scale_y = scale_y;
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
    x: f32,
    y: f32,
    scale_x: f32,
    scale_y: f32,
    rotation: f32,
    alpha: f32,
    is_dirty: bool,
}

impl GlobalTransform {
    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotation: 0.0,
            alpha: 1.0,
            is_dirty: false,
        }
    }

    pub fn x(&self) -> f32 {
        self.x
    }

    pub fn y(&self) -> f32 {
        self.y
    }

    pub fn scale_x(&self) -> f32 {
        self.scale_x
    }

    pub fn scale_y(&self) -> f32 {
        self.scale_y
    }

    pub fn rotation(&self) -> f32 {
        self.rotation
    }

    pub fn alpha(&self) -> f32 {
        self.alpha
    }

    pub fn update(&mut self, parent: &GlobalTransform, local: &LocalTransform) {
        if parent.is_dirty || local.is_dirty {
            let rx = local.x * parent.scale_x;
            let ry = local.y * parent.scale_y;

            let p_cos = parent.rotation.cos();
            let p_sin = parent.rotation.sin();

            self.scale_x = parent.scale_x * local.scale_x;
            self.scale_y = parent.scale_y * local.scale_y;

            let pivot_x = local.pivot_x * self.scale_x;
            let pivot_y = local.pivot_y * self.scale_y;

            let cos = local.rotation.cos();
            let sin = local.rotation.sin();

            self.x = parent.x + (rx * p_cos - ry * p_sin) - (pivot_x * cos - pivot_y * sin);
            self.y = parent.y + (rx * p_sin + ry * p_cos) - (pivot_x * sin + pivot_y * cos);

            self.rotation = parent.rotation + local.rotation;
            self.alpha = parent.alpha * local.alpha;

            self.is_dirty = true;
        }
    }
}
