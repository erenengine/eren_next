use std::collections::HashMap;

pub struct AssetsState<A> {
    pub ready: Vec<A>,
    pub pending: HashMap<A, &'static str>,
}

impl<A> AssetsState<A> {
    pub fn new() -> Self {
        Self {
            ready: Vec::new(),
            pending: HashMap::new(),
        }
    }
}
