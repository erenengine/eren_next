use super::sprite::SpriteDrawData;

pub struct GameState<AssetId> {
    pub sprite_draw_list: Vec<SpriteDrawData<AssetId>>,
}

impl<AssetId> GameState<AssetId> {
    pub fn new() -> Self {
        Self {
            sprite_draw_list: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.sprite_draw_list.clear();
    }
}
