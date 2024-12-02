use crate::persistent_avl_tree::PersistentAvlTree;
use crate::path_copy_avl::path_copy::CopyNode;

pub struct PathCopyAvl<Data: Ord> {
    data: Vec<Data>,
    node_arena: Vec<CopyNode>,
    root_nodes: Vec<Option<usize>>,
}

impl<Data: Ord> PathCopyAvl<Data> {
    fn get_data(&self, node: &CopyNode) -> &Data {
        &self.data[node.datum_ptr]
    }

    fn balance_and_clone(&self, path: Vec<usize>) -> Option<usize> {
        if path.len() <= 1 {
            path.last().copied()
        } else {
            todo!()
        }
    }
}

impl<Data: Ord> PersistentAvlTree for PathCopyAvl<Data> {
    type Data = Data;

    type Timestamp = usize;

    fn insert(&mut self, item: Self::Data) -> Self::Timestamp {
        // Allocation
        self.data.push(item);
        let datum_ptr = self.data.len() - 1;
        let item = &self.data[datum_ptr];

        self.node_arena.push(CopyNode {
            datum_ptr: datum_ptr,
            height: 0,
            left: None,
            right: None,
        });
        let new_node_ptr = self.node_arena.len() - 1;

        let mut path_ptr = self.root_nodes.last().and_then(|latest_time| *latest_time);

        let mut path = Vec::new();
        while let Some(ptr) = path_ptr {
            path.push(ptr);

            let node = &self.node_arena[ptr];
            let node_datum = self.get_data(node);

            if *item <= *node_datum {
                path_ptr = node.right;
            } else {
                path_ptr = node.left
            }
        }

        path.push(new_node_ptr);
        self.root_nodes.push(self.balance_and_clone(path));

        self.root_nodes.len() - 1
    }

    fn delete(&mut self, item: Self::Data) -> Option<Self::Timestamp> {
        todo!()
    }

    fn contains(&self, item: &Self::Data, timestamp: Self::Timestamp) -> bool {
        match self.predecessor(item, timestamp) {
            Some(predecessor) => *item == *predecessor,
            None => false,
        }
    }

    fn predecessor(&self, item: &Self::Data, timestamp: Self::Timestamp) -> Option<&Self::Data> {
        if timestamp >= self.root_nodes.len() {
            return None;
        }

        let mut node_ptr = self.root_nodes[timestamp];
        let mut inf: Option<&Self::Data> = None;

        while let Some(ptr) = node_ptr {
            let node = &self.node_arena[ptr];
            let datum = self.get_data(node);

            if *datum > *item {
                node_ptr = node.left;
            } else {
                inf = Some(datum);
                node_ptr = node.right;
            }
        }

        inf
    }

    fn successor(&self, item: &Self::Data, timestamp: Self::Timestamp) -> Option<&Self::Data> {
        if timestamp >= self.root_nodes.len() {
            return None;
        }

        let mut node_ptr = self.root_nodes[timestamp];
        let mut sup: Option<&Self::Data> = None;

        while let Some(ptr) = node_ptr {
            let node = &self.node_arena[ptr];
            let datum = self.get_data(node);

            if *datum < *item {
                node_ptr = node.right;
            } else {
                sup = Some(datum);
                node_ptr = node.right;
            }
        }

        sup
    }
}
