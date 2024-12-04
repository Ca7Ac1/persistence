use std::collections::HashMap;
use std::usize;

use crate::path_copy_avl::path_copy::CopyNode;
use crate::persistent_avl_tree::PersistentAvlTree;

pub struct PathCopyAvl<Data: Ord> {
    data: Vec<Data>,
    node_arena: Vec<CopyNode>,
    root_nodes: Vec<Option<usize>>,
}

impl<Data: Ord> PathCopyAvl<Data> {
    fn get_data(&self, node: &CopyNode) -> &Data {
        &self.data[node.datum_ptr]
    }

    fn get_node(&self, update_cache: &HashMap<usize, CopyNode>, node_ptr: usize) -> CopyNode {
        match update_cache.get(&node_ptr) {
            Some(node) => node.clone(),
            None => self.node_arena[node_ptr].clone(),
        }
    }

    fn modify(
        &self,
        update_cache: &mut HashMap<usize, CopyNode>,
        node_ptr: usize,
        height: u64,
        new_left_ptr: Option<usize>,
        new_right_ptr: Option<usize>,
    ) {
        update_cache.insert(
            node_ptr,
            self.get_node(&update_cache, node_ptr)
                .update(height, new_left_ptr, new_right_ptr),
        );
    }

    fn modify_height(
        &self,
        update_cache: &mut HashMap<usize, CopyNode>,
        node_ptr: usize,
        height: u64,
    ) {
        self.modify(
            update_cache,
            node_ptr,
            height,
            self.get_node(&update_cache, node_ptr).left,
            self.get_node(&update_cache, node_ptr).right,
        );
    }

    fn modify_node_left(
        &self,
        update_cache: &mut HashMap<usize, CopyNode>,
        node_ptr: usize,
        new_left_ptr: Option<usize>,
    ) {
        self.modify(
            update_cache,
            node_ptr,
            self.node_arena[node_ptr].height,
            new_left_ptr,
            self.get_node(&update_cache, node_ptr).right,
        );
    }

    fn modify_node_right(
        &self,
        update_cache: &mut HashMap<usize, CopyNode>,
        node_ptr: usize,
        new_right_ptr: Option<usize>,
    ) {
        self.modify(
            update_cache,
            node_ptr,
            self.node_arena[node_ptr].height,
            self.get_node(&update_cache, node_ptr).left,
            new_right_ptr,
        );
    }

    fn balance_and_clone(
        &self,
        update_cache: &mut HashMap<usize, CopyNode>,
        path: Vec<usize>,
    ) -> Option<usize> {
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
        let mut update_cache = HashMap::new();

        self.data.push(item);
        let datum_ptr = self.data.len() - 1;

        let item = &self.data[datum_ptr];

        let new_node_ptr = self.node_arena.len();
        update_cache.insert(
            new_node_ptr,
            CopyNode {
                datum_ptr: datum_ptr,
                height: 0,
                left: None,
                right: None,
            },
        );

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

        self.root_nodes
            .push(self.balance_and_clone(&mut update_cache, path));

        self.node_arena
            .push(update_cache.remove(&new_node_ptr).unwrap());
        self.node_arena
            .append(&mut update_cache.into_values().collect());

        self.root_nodes.len() - 1
    }

    fn delete(&mut self, item: &Self::Data) -> Option<Self::Timestamp> {
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
