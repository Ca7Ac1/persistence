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
        timestamp: u64,
        height: u64,
        new_right: Option<usize>,
        new_left: Option<usize>,
    ) -> CopyNode {
        CopyNode {
            datum_ptr: self.datum_ptr,
            height: height,
            right: new_right,
            left: new_left,
        }
    }

    pub(crate) fn update_left(
        &mut self,
        timestamp: u64,
        height: u64,
        new_left: Option<usize>,
    ) -> CopyNode {
        CopyNode {
            datum_ptr: self.datum_ptr,
            height: height,
            right: self.right,
            left: new_left,
        }
    }

    pub(crate) fn update_right(
        &mut self,
        timestamp: u64,
        height: u64,
        new_right: Option<usize>,
    ) -> CopyNode {
        CopyNode {
            datum_ptr: self.datum_ptr,
            height: height,
            right: new_right,
            left: self.left,
        }
    }
}