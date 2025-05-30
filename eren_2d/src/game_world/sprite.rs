use super::{state::GameState, update::Update};
use eren_core::math::Vec2;

pub struct SpriteDrawData<AssetId> {
    position: Vec2,
    scale: Vec2,
    rotation: f32,
    asset_id: AssetId,
}

pub struct Sprite<AssetId> {
    x: f32,
    y: f32,
    asset_id: AssetId,
}

impl<AssetId> Sprite<AssetId> {
    pub fn new(x: f32, y: f32, asset_id: AssetId) -> Self {
        Self { x, y, asset_id }
    }
}

impl<AssetId: Copy> Update<AssetId> for Sprite<AssetId> {
    fn update(&mut self, state: &mut GameState<AssetId>) {
        state.sprite_draw_list.push(SpriteDrawData {
            position: Vec2::new(self.x, self.y),
            scale: Vec2::new(1.0, 1.0),
            rotation: 0.0,
            asset_id: self.asset_id,
        });
    }
}
