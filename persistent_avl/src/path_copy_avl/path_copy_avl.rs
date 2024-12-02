use std::cmp::max;
use std::collections::HashMap;

use crate::persistent_avl_tree::PersistentAvlTree;
use crate::path_copy_avl::path_copy::CopyNode;

pub struct PathCopyAvl<Data: Ord> {
    data: Vec<Data>,
    node_arena: Vec<CopyNode>,
    root_nodes: Vec<Option<usize>>,
}

#[derive(Clone, Copy, Debug)]
enum Direction {
    LEFT,
    RIGHT,
}

impl<Data: Ord> PathCopyAvl<Data> {
    fn get_data(&self, node: &CopyNode) -> &Data {
        &self.data[node.datum_ptr]
    }

    fn get_datum_ptr(&self, node_ptr: usize) -> usize {
        self.node_arena[node_ptr].datum_ptr
    }

    fn get_height(&self, node_ptr: Option<usize>) -> u64 {
        match node_ptr {
            Some(node_ptr) => self.node_arena[node_ptr].height,
            None => 0,
        }
    }

    /// Given a traversal path in the previous time, which end can be determined to be after,
    /// allocates bottom-up new copies of the members of path (and also allocates end), which
    /// are balanced.
    fn build_copies_upward(&mut self, path: Vec<usize>, end: CopyNode) -> Option<usize> {
        if path.len() <= 0 {
            Some(self.allocate(end))
        } else {
            let batch_end = self.node_arena.len();
            let batch: HashMap<usize, CopyNode> = HashMap::new();
            let path_node = path.last().unwrap();

            todo!()
        }
    }

    fn allocate(&mut self, node: CopyNode) -> usize {
        self.node_arena.push(node);
        self.node_arena.len() - 1
    }

    fn get_or_alloc(&self, batch: &mut HashMap<usize, CopyNode>, ptr: usize) -> (usize, &CopyNode) {
        if ptr >= self.node_arena.len() {
            (ptr, &batch[&ptr])
        } else {

            &self.node_arena[ptr]
        }
    }

    fn rotate(&mut self, batch: &mut HashMap<usize, CopyNode>, prev_upper_ptr: usize, direction: Direction) {
        // prev refers to prior versions
        let prev_upper = self.get_or_alloc(batch, prev_upper_ptr);
        let prev_lower_ptr = match direction {
            Direction::LEFT => prev_upper.right,
            Direction::RIGHT => prev_upper.left,
        }.expect("There should be a node in the opposite direction of rotation");
        let prev_lower = self.get_or_alloc(batch, prev_lower_ptr);

        let curr_old_left = match direction {
            Direction::LEFT => prev_upper.left,
            Direction::RIGHT => node_to_promote.right,
        };
        let curr_old_right = match direction {
            Direction::LEFT => node_to_promote.left,
            Direction::RIGHT => prev_upper.right,
        };
        let curr_old_root = CopyNode {
            datum_ptr: prev_upper.datum_ptr,
            height: 1 + max(self.get_height(curr_old_left), self.get_height(curr_old_right)),
            left: curr_old_left,
            right: curr_old_right,
        };
        let curr_old_root_ptr = self.allocate(curr_old_root);

        // Compiler doesn't know that self.allocate doesn't modify prev_new_root
        let prev_new_root = &self.node_arena[prev_new_root_ptr];
        let curr_new_left = match direction {
            Direction::LEFT => Some(curr_old_root_ptr),
            Direction::RIGHT => prev_new_root.left,
        };
        let curr_new_right = match direction {
            Direction::LEFT => prev_new_root.right,
            Direction::RIGHT => Some(curr_old_root_ptr),
        };
        let curr_new_root = CopyNode {
            datum_ptr: prev_new_root.datum_ptr,
            height: 1 + max(self.get_height(curr_new_left), self.get_height(curr_new_right)),
            left: curr_new_left,
            right: curr_new_right,
        };
        curr_new_root
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

        let new_node = CopyNode {
            datum_ptr: datum_ptr,
            height: 0,
            left: None,
            right: None,
        };

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

        self.root_nodes.push(self.build_copies_upward(path, new_node));

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
