#[derive(Debug, Copy, Clone)]
pub(crate) struct CopyNode {
    pub(crate) datum_ptr: usize,
    pub(crate) height: u64,
    pub(crate) left: Option<usize>,
    pub(crate) right: Option<usize>,
}

impl CopyNode {
    pub(crate) fn update(
        &self,
        height: u64,
        new_left: Option<usize>,
        new_right: Option<usize>,
    ) -> CopyNode {
        CopyNode {
            datum_ptr: self.datum_ptr,
            height: height,
            left: new_left,
            right: new_right,
        }
    }
}
