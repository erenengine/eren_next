pub struct RenderItem {}

pub struct RenderList {
    items: Vec<RenderItem>,
}

impl RenderList {
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    pub fn add(&mut self, item: RenderItem) {
        self.items.push(item);
    }
}
