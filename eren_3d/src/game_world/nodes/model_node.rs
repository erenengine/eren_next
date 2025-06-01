use crate::game_world::{
    state::{GameState, RenderRequest},
    transform::{GlobalTransform, LocalTransform},
};

use super::game_node::GameNode;

pub struct ModelNode<MA> {
    pub transform: LocalTransform,
    global_transform: GlobalTransform,
    asset_id: MA,
}

impl<MA> ModelNode<MA> {
    pub fn new(asset_id: MA) -> Self {
        Self {
            transform: LocalTransform::new(),
            global_transform: GlobalTransform::new(),
            asset_id,
        }
    }
}

impl<MA: Copy> GameNode<MA> for ModelNode<MA> {
    fn update(
        &mut self,
        game_state: &mut GameState<MA>,
        parent_global_transform: &GlobalTransform,
    ) {
        self.global_transform
            .update(parent_global_transform, &mut self.transform);

        let (matrix, alpha) = self.global_transform.extract();

        game_state.render_requests.push(RenderRequest {
            matrix,
            alpha,
            model_asset_id: self.asset_id,
        });
    }
}
