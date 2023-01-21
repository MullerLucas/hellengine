#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ResourceHandle {
    pub idx: usize,
}

impl ResourceHandle {
    pub const INVALID: ResourceHandle = Self::new(usize::MAX);

    pub const fn new(idx: usize) -> Self {
        Self {
            idx
        }
    }
}
