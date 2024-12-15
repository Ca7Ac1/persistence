pub(crate) struct OptAVLNode<'a, Timestamp: Ord> {
    pub(crate) datum_ptr: usize,
    pub(crate) height: u64,
    pub(crate) timestamp: &'a Timestamp, // Remark: For data, it is reasonable for the tree to own the data. Not so much for timestamps.
    pub(crate) l1: Option<usize>,
    pub(crate) r1: Option<usize>,
    pub(crate) l2: Option<usize>,
    pub(crate) r2: Option<usize>,
}

impl<Timestamp: Ord> OptAVLNode<'_, Timestamp> {
    pub(crate) fn get_left(
        &self,
        timestamp: &Timestamp,
    ) -> Option<usize> {
        if timestamp < self.timestamp {
            self.l1
        } else {
            self.l2
        }
    }

    pub(crate) fn get_right(
        &self,
        timestamp: &Timestamp,
    ) -> Option<usize> {
        if timestamp < self.timestamp {
            self.r1
        } else {
            self.r2
        }
    }

    /// These functions are helpers for rotations.

    /// Ignores the presence of any pre-existing pointers.
    /// Sets l1 if timestamp < self.timestamp, l2 if timestamp >= self.timestamp
    pub(crate) fn modify_left_with_replacement(
        &mut self,
        new_ptr: Option<usize>,
        timestamp: &Timestamp,
    ) {
        if timestamp < self.timestamp {
            self.l1 = new_ptr;
        } else {
            self.l2 = new_ptr;
        }
    }

    /// Ignores the presence of any pre-existing pointers.
    /// Sets r1 if timestamp < self.timestamp, r2 if timestamp >= self.timestamp
    pub(crate) fn modify_right_with_replacement(
        &mut self,
        new_ptr: Option<usize>,
        timestamp: &Timestamp,
    ) {
        if timestamp < self.timestamp {
            self.r1 = new_ptr;
        } else {
            self.r2 = new_ptr;
        }
    }

    fn duplicate_with_new_ptrs<'a>(
        &self,
        timestamp: &'a Timestamp,
        l2: Option<usize>,
        r2: Option<usize>,
    ) -> OptAVLNode<'a, Timestamp> {
        // Note that the lifetime of self need not match
        // the lifetime of the duplicated node.
        // Of course, this eventually ends up not mattering
        // because all nodes will have the same lifetime as the
        // AVL tree/arena itself.
        OptAVLNode {
            datum_ptr: self.datum_ptr,
            height: self.height,
            timestamp,
            l1: self.l2,
            r1: self.r2,
            l2,
            r2,
        }
    }

    /// Remark: if you ever want to update both pointers
    /// at once, you need to add a third helper.
    ///
    /// If you need to modify both the left and right pointer,
    /// you don't want to handle generating two duplicates,
    /// so you need one that simultaneously updates both.
    /// If you only want to modify one of them, you don't want
    /// to incorrectly duplicate when you don't need to.

    /// These functions should only be used to perform the actual
    /// insertion; helpers for rotation appear above.

    /// Sets l2 if timestamp >= self.timestamp and l2 == None.
    /// Panics if timestamp < self.timestamp; this is undefined
    /// behavior.
    /// Otherwise, returns a new node with l1 := l2, r1 := r2, l2 := new_ptr.
    pub(crate) fn modify_left_or_duplicate<'a>(
        &mut self,
        new_ptr: Option<usize>,
        timestamp: &'a Timestamp,
    ) -> Option<OptAVLNode<'a, Timestamp>> {
        debug_assert!(
            timestamp >= self.timestamp,
            "Attempted to modify l1 in previous time on insertion/deletion"
        );
        if let None = self.l2 {
            self.l2 = new_ptr;
            None
        } else {
            Some(self.duplicate_with_new_ptrs(timestamp, new_ptr, None))
        }
    }

    /// Sets r2 if timestamp >= self.timestamp and r2 == None.
    /// Panics if timestamp < self.timestamp; this is undefined
    /// behavior.
    /// Otherwise, returns a new node with l1 := l2, r1 := r2, r2 := new_ptr.
    pub(crate) fn modify_right_or_duplicate<'a>(
        &mut self,
        new_ptr: Option<usize>,
        timestamp: &'a Timestamp,
    ) -> Option<OptAVLNode<'a, Timestamp>> {
        debug_assert!(
            timestamp >= self.timestamp,
            "Attempted to modify r1 in previous time on insertion/deletion"
        );
        if let None = self.r2 {
            self.r2 = new_ptr;
            None
        } else {
            Some(self.duplicate_with_new_ptrs(timestamp, None, new_ptr))
        }
    }
}
