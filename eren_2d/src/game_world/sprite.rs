use super::{
    state::{GameState, RenderRequest},
    transform::{GlobalTransform, LocalTransform},
    update::Update,
};

pub struct Sprite<SA> {
    pub local_transform: LocalTransform,
    global_transform: GlobalTransform,
    asset_id: SA,
}

impl<SA> Sprite<SA> {
    pub fn new(x: f32, y: f32, asset_id: SA) -> Self {
        Self {
            local_transform: LocalTransform::new(x, y),
            global_transform: GlobalTransform::new(),
            asset_id,
        }
    }
}

impl<SA: Copy> Update<SA> for Sprite<SA> {
    fn update(
        &mut self,
        game_state: &mut GameState<SA>,
        parent_global_transform: &GlobalTransform,
    ) {
        self.global_transform
            .update(parent_global_transform, &self.local_transform);
        game_state.render_requests.push(RenderRequest {
            x: self.global_transform.x(),
            y: self.global_transform.y(),
            sprite_asset_id: self.asset_id,
        });
    }
}
