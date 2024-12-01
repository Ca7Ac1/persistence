use std::cmp::max;

use crate::fat_node::*;
use crate::persistent_avl_tree::PersistentAvlTree;
use crate::timestamp::get_time;

pub struct FatNodeAvl<Data: Ord> {
    node_arena: Vec<FatNode<Data>>,
    root_nodes: Vec<RootNode>,
    last_time: u64,
}

#[derive(Clone, Copy, Debug)]
enum Direction {
    LEFT, RIGHT
}

impl<Data: Ord> FatNodeAvl<Data> {
    fn get_height(&self, node_ptr: Option<usize>) -> u64 {
        match node_ptr {
            Some(node_ptr) => self.node_arena[node_ptr].height,
            None => 0,
        }
    }

    /// Rotating mutates affected fat nodes
    /// by adding a new entry with new pointers with the provided timestamp.
    /// Or if it already exists, modifying that entry
    fn rotate_left(&mut self, old_root_ptr: usize, timestamp: u64) -> usize {
        let old_root = &self.node_arena[old_root_ptr];

        let children = old_root
            .children
            .last()
            .expect("Failed to find children for left rotation");
        let old_root_left = children.left;
        let old_root_right = children.right;

        let new_root_ptr = old_root_right.expect("Failed to find right child for left rotation");
        let new_root = &self.node_arena[new_root_ptr];

        let children = new_root
            .children
            .last()
            .expect("Failed to find right grandchildren for left rotation");
        let new_root_left = children.left;
        let new_root_right = children.right;

        let old_root_height = max(
            self.get_height(old_root_left),
            self.get_height(new_root_left),
        );
        let new_root_height = max(
            old_root_height,
            self.get_height(new_root_right),
        );

        self.node_arena[old_root_ptr].modify_right(timestamp, old_root_height, new_root_left);
        self.node_arena[new_root_ptr].modify_left(timestamp, new_root_height, Some(old_root_ptr));

        new_root_ptr
    }

    fn rotate_right(&mut self, old_root_ptr: usize, timestamp: u64) -> usize {
        let old_root = &self.node_arena[old_root_ptr];

        let children = old_root
            .children
            .last()
            .expect("Failed to find children for right rotation");
        let old_root_left = children.left;
        let old_root_right = children.right;

        let new_root_ptr = old_root_left.expect("Failed to find left child for right rotation");
        let new_root = &self.node_arena[new_root_ptr];

        let children = new_root
            .children
            .last()
            .expect("Failed to find left grandchildren for right rotation");
        let new_root_left = children.left;
        let new_root_right = children.right;

        let old_root_height = max(
            self.get_height(old_root_right),
            self.get_height(new_root_right),
        );
        let new_root_height = max(
            old_root_height,
            self.get_height(new_root_left),
        );

        self.node_arena[old_root_ptr].modify_left(timestamp, old_root_height, new_root_right);
        self.node_arena[new_root_ptr].modify_right(timestamp, new_root_height, Some(old_root_ptr));

        new_root_ptr
    }

    /// Given the traversal path to an inserted/deleted node starting from the root
    /// and ending at the parent of the inserted/deleted node, updates the heights.
    fn update_heights(&mut self, modification_path: &Vec<usize>) {
        modification_path
            .iter()
            .rev()
            .for_each(|node_ptr| {
                let latest_children_maybe = self.node_arena[*node_ptr].children.last();
                let max_latest_children_height = match latest_children_maybe {
                    None => 0,
                    Some(latest_children) => {
                        let left_height = latest_children.left.map_or(0, |left_child| self.node_arena[left_child].height);
                        let right_height = latest_children.right.map_or(0, |right_child| self.node_arena[right_child].height);
                        max(left_height, right_height)
                    }
                };
                self.node_arena[*node_ptr].height = max_latest_children_height + 1;
            });
    }

    /// Returns the right subtree height minus the left subtree height of a node
    fn balance_factor(&self, node_ptr: usize) -> i32 {
        let latest_children_maybe = self.node_arena[node_ptr].children.last();
        match latest_children_maybe {
            None => 0,
            Some(latest_children) => {
                let left_height = latest_children.left.map_or(0, |left_child| self.node_arena[left_child].height);
                let right_height = latest_children.right.map_or(0, |right_child| self.node_arena[right_child].height);
                right_height as i32 - left_height as i32
            }
        }
    }

    /// Given the traversal path with directions to inserted/deleted node (root to parent of inserted/deleted node),
    /// Balances the lowest height unbalanced node
    /// Returns the index into the path at which a balance occurred on that node pointer, or None if it did not occur
    fn balance_lowest(&mut self, modification_path: &Vec<usize>, direction_path: &Vec<Direction>, timestamp: u64) -> Option<usize> {
        for (index, (node_and_child_ptrs, dir_after_after)) in Iterator::zip(0..modification_path.len() - 1, Iterator::zip(modification_path.windows(2), direction_path.iter().skip(1))) {
            let node_ptr = node_and_child_ptrs[0];
            let child_ptr = node_and_child_ptrs[1];
            let b = self.balance_factor(node_ptr);
            if b >= 2 {
                // RL
                if let Direction::LEFT = dir_after_after {
                    self.rotate_right(child_ptr, timestamp);
                }
                // RR
                self.rotate_left(node_ptr, timestamp);
            } else if b <= -2 {
                // LR
                if let Direction::RIGHT = dir_after_after {
                    self.rotate_left(child_ptr, timestamp);
                }
                // LL
                self.rotate_right(node_ptr, timestamp);
            }
            return Some(index);
        }
        None
    }
}

impl<Data: Ord> PersistentAvlTree for FatNodeAvl<Data> {
    type Data = Data;
    type Timestamp = u64;

    fn insert(&mut self, item: Self::Data) -> Self::Timestamp {
        // Allocation
        self.node_arena.push(FatNode {
            datum: item,
            height: 1,
            children: Vec::new(),
        });
        let new_node_ptr = self.node_arena.len() - 1;
        let item = &self.node_arena[new_node_ptr].datum;

        // Insertion
        let latest_root_meta_maybe = self.root_nodes.last();
        let latest_root_maybe =
            latest_root_meta_maybe.and_then(|latest_root_meta| latest_root_meta.root);
        match latest_root_maybe {
            Some(latest_root_ptr) => {
                // Traversal
                let mut insertion_path = Vec::new();
                let mut direction_path = Vec::new();
                let mut curr_node_ptr_maybe = Some(latest_root_ptr);
                while let Some(curr_node_ptr) = curr_node_ptr_maybe {
                    insertion_path.push(curr_node_ptr);

                    let curr_node = &self.node_arena[curr_node_ptr];
                    let latest_children_maybe = curr_node.children.last();
                    if *item >= curr_node.datum {
                        direction_path.push(Direction::RIGHT);
                        curr_node_ptr_maybe =
                            latest_children_maybe.and_then(|latest_children| latest_children.right);
                    } else {
                        direction_path.push(Direction::LEFT);
                        curr_node_ptr_maybe =
                            latest_children_maybe.and_then(|latest_children| latest_children.left);
                    }
                }

                // Balance
                self.update_heights(&insertion_path);
            }
            None => self.root_nodes.push(RootNode {
                timestamp: self.last_time,
                root: Some(new_node_ptr),
            }),
        }

        self.last_time += 1;
        self.last_time - 1
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
        let mut node_ptr = get_time(&self.root_nodes, &timestamp)?.root;
        let mut inf: Option<&Self::Data> = None;

        while let Some(current_ptr) = node_ptr {
            let node = &self.node_arena[current_ptr];
            let children = get_time(&node.children, &timestamp);

            match children {
                Some(children) => {
                    if node.datum > *item {
                        node_ptr = children.left;
                    } else {
                        inf = Some(&node.datum);
                        node_ptr = children.right;
                    }
                }
                None => break,
            }
        }

        inf
    }

    fn successor(&self, item: &Self::Data, timestamp: Self::Timestamp) -> Option<&Self::Data> {
        let mut node_ptr = get_time(&self.root_nodes, &timestamp)?.root;
        let mut sup: Option<&Self::Data> = None;

        while let Some(current_ptr) = node_ptr {
            let node = &self.node_arena[current_ptr];
            let children = get_time(&node.children, &timestamp);

            match children {
                Some(children) => {
                    if node.datum < *item {
                        node_ptr = children.right;
                    } else {
                        sup = Some(&node.datum);
                        node_ptr = children.left;
                    }
                }
                None => break,
            }
        }

        sup
    }
}
