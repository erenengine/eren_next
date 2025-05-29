pub struct AssetBundle {
    assets: Vec<(String, String)>,
}

impl AssetBundle {
    pub fn new(assets: Vec<(&str, &str)>) -> Self {
        let assets = assets
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        Self { assets }
    }

    pub fn add_asset(&mut self, id: &str, path: &str) {
        self.assets.push((id.to_string(), path.to_string()));
    }

    pub fn is_loaded(&self) -> bool {
        false
    }
}
