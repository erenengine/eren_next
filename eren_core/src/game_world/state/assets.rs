pub struct AssetsState<A> {
    ready: Vec<A>,
    pending: Vec<A>,
}

impl<A> AssetsState<A> {
    pub fn new() -> Self {
        Self {
            ready: Vec::new(),
            pending: Vec::new(),
        }
    }
}
