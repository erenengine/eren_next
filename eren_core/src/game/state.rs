pub struct GameState {
    pub required_assets: Vec<String>,
    pub loaded_assets: Vec<String>,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            required_assets: Vec::new(),
            loaded_assets: Vec::new(),
        }
    }

    pub fn is_asset_loaded(&mut self, asset_id: String) -> bool {
        if self.loaded_assets.contains(&asset_id) {
            return true;
        }
        if !self.required_assets.contains(&asset_id) {
            self.required_assets.push(asset_id.clone());
        }
        false
    }
}
