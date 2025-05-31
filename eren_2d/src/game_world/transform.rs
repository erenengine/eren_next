pub struct LocalTransform {
    x: f32,
    y: f32,
    is_dirty: bool,
}

impl LocalTransform {
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            is_dirty: true,
        }
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

    pub fn x(&self) -> f32 {
        self.x
    }

    pub fn y(&self) -> f32 {
        self.y
    }
}

pub struct GlobalTransform {
    x: f32,
    y: f32,
    is_dirty: bool,
}

impl GlobalTransform {
    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            is_dirty: false,
        }
    }

    pub fn x(&self) -> f32 {
        self.x
    }

    pub fn y(&self) -> f32 {
        self.y
    }

    pub fn update(&mut self, parent: &GlobalTransform, local: &LocalTransform) {
        if parent.is_dirty || local.is_dirty {
            self.x = parent.x + local.x;
            self.y = parent.y + local.y;
            self.is_dirty = true;
        }
    }
}
