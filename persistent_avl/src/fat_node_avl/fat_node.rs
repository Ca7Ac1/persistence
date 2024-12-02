use crate::timestamp::*;

pub(crate) struct ChildrenAtTime {
    pub(crate) timestamp: u64,
    pub(crate) left: Option<usize>,
    pub(crate) right: Option<usize>,
}

impl TimestampSupplier for ChildrenAtTime {
    type Timestamp = u64;

    fn get_timestamp(&self) -> &Self::Timestamp {
        &self.timestamp
    }
}

pub(crate) struct FatNode<Data: Ord> {
    pub(crate) datum: Data,
    pub(crate) height: u64,
    pub(crate) children: Vec<ChildrenAtTime>,
}

// All modifications to a FatNode assume that the given 
// timestamp is >= the timestamp of latest child
impl<Data: Ord> FatNode<Data> {
    pub(crate) fn modify_left(&mut self, timestamp: u64, new_left: Option<usize>) {
        match self.children.last_mut().filter(|last_children| last_children.timestamp == timestamp) {
            // When last children exist & match your timestamp, just mutate instead
            Some(last_children) => last_children.left = new_left,
            None => self.children.push(ChildrenAtTime {
                timestamp: timestamp,
                left: new_left,
                right: self.children.last().and_then(|children| children.right),
            }),
        };
    }

    pub(crate) fn modify_right(&mut self, timestamp: u64, new_right: Option<usize>) {
        match self.children.last_mut().filter(|last_children| last_children.timestamp == timestamp) {
            // When last children exist & match your timestamp, just mutate instead
            Some(last_children) => last_children.right = new_right,
            None => self.children.push(ChildrenAtTime {
                timestamp: timestamp,
                left: self.children.last().and_then(|children| children.left),
                right: new_right,
            }),
        };
    }

    pub(crate) fn set_height(&mut self, height: u64) {
        self.height = height;
    }

}

pub(crate) struct RootNode {
    pub(crate) timestamp: u64,
    pub(crate) root: Option<usize>,
}

impl TimestampSupplier for RootNode {
    type Timestamp = u64;

    fn get_timestamp(&self) -> &Self::Timestamp {
        &self.timestamp
    }
}
