use super::{
    state::{GameState, RenderRequest},
    update::Update,
};

pub struct Sprite<SA> {
    x: f32,
    y: f32,
    asset_id: SA,
}

impl<SA> Sprite<SA> {
    pub fn new(x: f32, y: f32, asset_id: SA) -> Self {
        Self { x, y, asset_id }
    }
}

impl<SA: Copy> Update<SA> for Sprite<SA> {
    fn update(&mut self, state: &mut GameState<SA>) {
        state.render_requests.push(RenderRequest {
            x: self.x,
            y: self.y,
            sprite_asset_id: self.asset_id,
        });
    }
}
