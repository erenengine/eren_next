use crate::render::RenderList;

pub struct GameContext {
    pub render_list: RenderList,
}

impl GameContext {
    pub fn new() -> Self {
        Self {
            render_list: RenderList::new(),
        }
    }
}
