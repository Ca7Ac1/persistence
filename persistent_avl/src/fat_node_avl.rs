use crate::persistent_avl_tree::PersistentAvlTree;

struct DataTime<Data: Ord, Timestamp: Ord> {
    timestamp: Timestamp,
    left: Option<FatNode<Data, Timestamp>>,
    right: Option<FatNode<Data, Timestamp>>,
}

// Invariant: Children are in sorted order by timestamp
struct FatNode<Data: Ord, Timestamp: Ord> {
    datum: Data,
    children: Vec<DataTime<Data, Timestamp>>,
}

impl<Data: Ord, Timestamp: Ord> FatNode<Data, Timestamp> {
    fn get_time(&self, time: Timestamp) -> Option<&DataTime<Data, Timestamp>> {
        if self.children.is_empty() {
            return None;
        }

        let mut low: usize = 0;
        let mut high: usize = self.children.len() - 1;
        while low < high {
            let mid: usize = (low + high + 1) / 2;
            let mid_time: &Timestamp = &self.children[mid].timestamp;

            if *mid_time > time {
                high = mid - 1;
            } else {
                low = mid;
            }
        }

        if self.children[low].timestamp <= time {
            Some(&self.children[low])
        } else {
            None
        }
    }
}

pub struct FatNodeAvl<T: Ord> {
    head: Option<FatNode<T, u64>>,
}

impl<T: Ord> FatNodeAvl<T> {}

impl<T: Ord> PersistentAvlTree for FatNodeAvl<T> {
    type Data = T;
    type Timestamp = u64;

    fn insert(&mut self, item: Self::Data) -> Self::Timestamp {
        todo!()
    }

    fn delete(&mut self, item: Self::Data) -> Self::Timestamp {
        todo!()
    }

    fn contains(&self, item: &Self::Data, timestamp: Self::Timestamp) -> bool {
        match self.predecessor(item, timestamp) {
            Some(predecessor) => *item == *predecessor,
            None => false,
        }
    }

    fn predecessor(&self, item: &Self::Data, timestamp: Self::Timestamp) -> Option<&Self::Data> {
        let mut root: Option<&FatNode<T, u64>> = self.head.as_ref();
        let mut inf: Option<&Self::Data> = None;
        while let Some(node) = root {
            let children: &DataTime<T, u64> = node.get_time(timestamp)?;

            if node.datum > *item {
                root = children.left.as_ref();
            } else {
                inf = Some(&node.datum);
                root = children.right.as_ref();
            }
        };

        inf
    }

    fn successor(&self, item: &Self::Data, timestamp: Self::Timestamp) -> Option<&Self::Data> {
        let mut root: Option<&FatNode<T, u64>> = self.head.as_ref();
        let mut sup: Option<&Self::Data> = None;
        while let Some(node) = root {
            let children: &DataTime<T, u64> = node.get_time(timestamp)?;

            if node.datum < *item {
                root = children.right.as_ref();
            } else {
                sup = Some(&node.datum);
                root = children.left.as_ref();
            }
        };

        sup
    }
}
