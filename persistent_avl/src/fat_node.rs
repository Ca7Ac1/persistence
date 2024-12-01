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

impl<Data: Ord> FatNode<Data> {
    pub(crate) fn modify_data(&mut self, new_data: ChildrenAtTime, height: u64) {
        self.children.push(new_data);

        self.height = height;
    }

    pub(crate) fn modify(&mut self, timestamp: u64, height: u64, new_right: Option<usize>, new_left: Option<usize>) {
        self.children.push(ChildrenAtTime { 
            timestamp: timestamp, 
            left: new_left, 
            right: new_right 
        });

        self.height = height;
    }

    pub(crate) fn modify_left(&mut self, timestamp: u64, height: u64, new_left: Option<usize>) {
        self.children.push(ChildrenAtTime {
            timestamp: timestamp,
            left: new_left,
            right: self.children.last().and_then(|children| children.right),
        });

        self.height = height;
    }

    pub(crate) fn modify_right(&mut self, timestamp: u64, height: u64, new_right: Option<usize>) {
        self.children.push(ChildrenAtTime {
            timestamp: timestamp,
            left: self.children.last().and_then(|children| children.left),
            right: new_right,
        });

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
