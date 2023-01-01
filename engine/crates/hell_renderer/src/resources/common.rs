#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ResourceHandle {
    pub id: usize,
}

impl ResourceHandle {
    pub const INVALID: ResourceHandle = Self::new(usize::MAX);

    pub const fn new(id: usize) -> Self {
        Self {
            id
        }
    }
}
