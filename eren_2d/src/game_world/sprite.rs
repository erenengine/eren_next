use super::{
    game_node::GameNode,
    state::{GameState, RenderRequest},
    transform::{GlobalTransform, LocalTransform},
};

pub struct Sprite<SA> {
    pub transform: LocalTransform,
    global_transform: GlobalTransform,
    asset_id: SA,
}

impl<SA> Sprite<SA> {
    pub fn new(asset_id: SA) -> Self {
        Self {
            transform: LocalTransform::new(),
            global_transform: GlobalTransform::new(),
            asset_id,
        }
    }
}

impl<SA: Copy> GameNode<SA> for Sprite<SA> {
    fn update(
        &mut self,
        game_state: &mut GameState<SA>,
        parent_global_transform: &GlobalTransform,
    ) {
        self.global_transform
            .update(parent_global_transform, &self.transform);

        game_state.render_requests.push(RenderRequest {
            matrix: self.global_transform.matrix(),
            alpha: self.global_transform.alpha(),
            sprite_asset_id: self.asset_id,
        });
    }
}
