use std::cmp::max;

use crate::fat_node_avl::fat_node::{FatNode, RootNode};
use crate::persistent_avl_tree::PersistentAvlTree;
use crate::timestamp::get_time;

pub struct FatNodeAvl<Data: Ord> {
    node_arena: Vec<FatNode<Data>>,
    root_nodes: Vec<RootNode>,
    last_time: u64,
}

enum RotationDirection {
    LEFT,
    RIGHT,
}

impl<Data: Ord> FatNodeAvl<Data> {
    fn modify_root(&mut self, new_node_ptr: Option<usize>, timestamp: u64) {
        if let None = self
            .root_nodes
            .last()
            .filter(|root_node| root_node.root == new_node_ptr)
        {
            self.root_nodes.push(RootNode {
                timestamp,
                root: new_node_ptr,
            });
        }
    }

    /// Calculates the height of a node assuming its childrens heights are set
    fn set_height(&mut self, node_ptr: usize) {
        let children = self.node_arena[node_ptr].children.last();

        let child_height = match children {
            None => 0,
            Some(children) => {
                let left_height = children
                    .left
                    .map_or(0, |left_child| self.node_arena[left_child].height);
                let right_height = children
                    .right
                    .map_or(0, |right_child| self.node_arena[right_child].height);

                max(left_height, right_height)
            }
        };

        self.node_arena[node_ptr].height = child_height + 1;
    }

    fn balance_node(&mut self, timestamp: u64, node_ptr: usize) -> usize {
        let node = &self.node_arena[node_ptr];
        let b = self.balance_factor(node_ptr);
        if b <= -2 {
            let left_child_ptr = node.children.last().unwrap().left.unwrap();
            // LR
            if self.balance_factor(left_child_ptr) >= 1 {
                let new_left_child_ptr = self.rotate(timestamp, left_child_ptr, RotationDirection::LEFT);
                node.modify_left(timestamp, Some(new_left_child_ptr));
                self.set_height(node_ptr);
            }
            // LL & LR
            self.rotate(timestamp, node_ptr, RotationDirection::RIGHT)
        } else if b >= 2 {
            let right_child_ptr = node.children.last().unwrap().right.unwrap();
            // RL
            if self.balance_factor(right_child_ptr) <= -1 {
                let new_right_child_ptr = self.rotate(timestamp, right_child_ptr, RotationDirection::RIGHT);
                node.modify_right(timestamp, Some(new_right_child_ptr));
                self.set_height(node_ptr);
            }
            // RL & RR
            self.rotate(timestamp, node_ptr, RotationDirection::LEFT)
        } else {
            node_ptr
        }
    }

    fn get_height(&self, node_ptr: Option<usize>) -> u64 {
        match node_ptr {
            Some(ptr) => self.node_arena[ptr].height,
            None => 0,
        }
    }

    fn balance_factor(&self, node_ptr: usize) -> i32 {
        let node = &self.node_arena[node_ptr];
        match node.children.last() {
            Some(cat) => self.get_height(cat.right) as i32 - self.get_height(cat.left) as i32,
            None => 0,
        }
    }

    /// Precondition: the child in the opposite direction of rotation exists.
    /// Rotates the tree in the given direction, updating pointers via pushing to node.
    fn rotate(&mut self, timestamp: u64, node_ptr: usize, direction: RotationDirection) -> usize {
        let node = &self.node_arena[node_ptr];
        let child_ptr = match direction {
            // See the precondition for the unwrap
            RotationDirection::LEFT => node.children.last().unwrap().right.unwrap(),
            RotationDirection::RIGHT => node.children.last().unwrap().left.unwrap(),
        };
        let child_node = &self.node_arena[child_ptr];

        let lower_left = match direction {
            RotationDirection::LEFT => node.children.last().and_then(|cat| cat.left),
            RotationDirection::RIGHT => child_node.children.last().and_then(|cat| cat.right),
        };
        let lower_right = match direction {
            RotationDirection::LEFT => child_node.children.last().and_then(|cat| cat.left),
            RotationDirection::RIGHT => node.children.last().and_then(|cat| cat.right),
        };

        let upper_left = match direction {
            RotationDirection::LEFT => Some(node_ptr),
            RotationDirection::RIGHT => child_node.children.last().and_then(|cat| cat.left),
        };
        let upper_right = match direction {
            RotationDirection::LEFT => child_node.children.last().and_then(|cat| cat.right),
            RotationDirection::RIGHT => Some(node_ptr),
        };

        // Input node is now lower
        let node = &mut self.node_arena[node_ptr];
        node.modify_left(timestamp, lower_left);
        node.modify_right(timestamp, lower_right);
        self.set_height(node_ptr);

        // Child node is now upper
        let child_node = &mut self.node_arena[node_ptr];
        child_node.modify_left(timestamp, upper_left);
        child_node.modify_right(timestamp, upper_right);
        self.set_height(child_ptr);

        child_ptr
    }

    /// Calculates the heights and rebalances the tree up `path`
    ///
    /// Returns the element at the root of `path` after modifications are complete
    fn balance(&mut self, timestamp: u64, path: &Vec<usize>) -> Option<usize> {
        let mut child_ptr = *path.last()?;

        path.iter().rev().skip(1).for_each(|&parent_ptr| {
            self.set_height(child_ptr);
            child_ptr = self.balance_node(timestamp, child_ptr);

            if self.node_arena[parent_ptr].datum <= self.node_arena[child_ptr].datum {
                self.node_arena[parent_ptr].modify_left(timestamp, Some(child_ptr));
            } else {
                self.node_arena[parent_ptr].modify_right(timestamp, Some(child_ptr));
            }

            child_ptr = parent_ptr;
        });

        self.set_height(child_ptr);
        Some(self.balance_node(timestamp, child_ptr))
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
        let root = self.root_nodes.last().and_then(|root_node| root_node.root);

        match root {
            Some(mut parent_ptr) => {
                let mut path = vec![parent_ptr];

                while let Some(child_ptr) =
                    self.node_arena[parent_ptr]
                        .children
                        .last()
                        .and_then(|children| {
                            if *item <= self.node_arena[parent_ptr].datum {
                                children.left
                            } else {
                                children.right
                            }
                        })
                {
                    path.push(child_ptr);
                    parent_ptr = child_ptr;
                }

                if *item <= self.node_arena[parent_ptr].datum {
                    self.node_arena[parent_ptr].modify_left(self.last_time, Some(new_node_ptr));
                } else {
                    self.node_arena[parent_ptr].modify_right(self.last_time, Some(new_node_ptr));
                }

                let new_root = self.balance(self.last_time, &path);
                self.modify_root(new_root, self.last_time);
            }
            None => self.modify_root(Some(new_node_ptr), self.last_time),
        }

        self.last_time += 1;
        self.last_time - 1
    }

    fn delete(&mut self, item: &Self::Data) -> Option<Self::Timestamp> {
        let mut parent_ptr = None;
        let mut child_ptr = self
            .root_nodes
            .last()
            .and_then(|root_node| root_node.root)?;

        // Path keeping track of all modified nodes in order
        let mut path = Vec::new();

        // Traverse to node to delete
        while self.node_arena[child_ptr].datum != *item {
            path.push(child_ptr);
            parent_ptr = Some(child_ptr);

            let children = self.node_arena[child_ptr].children.last()?;

            child_ptr = if self.node_arena[child_ptr].datum < *item {
                children.right?
            } else {
                children.left?
            };
        }

        let children_of_deleted = self.node_arena[child_ptr].children.last();

        let left_of_deleted = children_of_deleted.and_then(|children| children.left);
        let right_of_deleted = children_of_deleted.and_then(|children| children.left);

        // match against both children of deleted node existing
        match left_of_deleted.zip(right_of_deleted) {
            Some((_, right_subtree_ptr)) => {
                match self.node_arena[right_subtree_ptr]
                    .children
                    .last()
                    .and_then(|children| children.left)
                {
                    // If the left child of our right subtree exists we find the
                    // successor of our deleted node. Note that this successor
                    // necessarily doesn't have a right child, as otherwise that
                    // would be our successor. We replace our deleted node with its
                    // successor, and give the successors right child to its parent.
                    Some(mut sup_ptr) => {
                        let mut sup_parent_ptr = right_subtree_ptr;
                        let mut displaced_path = vec![sup_parent_ptr];

                        while let Some(lesser) = self.node_arena[sup_ptr]
                            .children
                            .last()
                            .and_then(|children| children.left)
                        {
                            sup_parent_ptr = sup_ptr;
                            displaced_path.push(sup_parent_ptr);

                            sup_ptr = lesser;
                        }

                        // Our path will be up to the deleted node, then the next 
                        // node will be our successor, and then the next nodes will  
                        // be the the path down to where our successor was located.
                        path.push(sup_ptr);
                        path.append(&mut displaced_path);

                        let right_of_sup = self.node_arena[sup_ptr]
                            .children
                            .last()
                            .and_then(|children| children.right);
                        self.node_arena[sup_parent_ptr].modify_left(self.last_time, right_of_sup);

                        self.node_arena[sup_ptr].modify_left(self.last_time, left_of_deleted);
                        self.node_arena[sup_ptr].modify_right(self.last_time, right_of_deleted);

                        if let Some(parent_ptr) = parent_ptr {
                            if self.node_arena[parent_ptr].children.last().unwrap().left
                                == Some(child_ptr)
                            {
                                self.node_arena[parent_ptr]
                                    .modify_left(self.last_time, Some(sup_ptr));
                            } else {
                                self.node_arena[parent_ptr]
                                    .modify_right(self.last_time, Some(sup_ptr));
                            }
                        };
                    },

                    // If the left child of our right subtree does not exist,
                    // we give the left child of our deleted node to the right 
                    // subtree, and replace our deleted node with its right child.
                    None => {
                        self.node_arena[right_subtree_ptr]
                            .modify_left(self.last_time, left_of_deleted);

                        if let Some(parent_ptr) = parent_ptr {
                            if self.node_arena[parent_ptr].children.last().unwrap().left
                                == Some(child_ptr)
                            {
                                self.node_arena[parent_ptr]
                                    .modify_left(self.last_time, Some(right_subtree_ptr));
                            } else {
                                self.node_arena[parent_ptr]
                                    .modify_right(self.last_time, Some(right_subtree_ptr));
                            }
                        };
                    }
                }
            }
            // If the deleted node has a single child then we replace it with that child.
            // Otherwise if the deleted node has no children we remove it without replacement.
            None => {
                let new_child = if let None = left_of_deleted {
                    right_of_deleted
                } else if let None = right_of_deleted {
                    left_of_deleted
                } else {
                    None
                };

                if let Some(parent_ptr) = parent_ptr {
                    if self.node_arena[parent_ptr].children.last().unwrap().left == Some(child_ptr)
                    {
                        self.node_arena[parent_ptr].modify_left(self.last_time, new_child);
                    } else {
                        self.node_arena[parent_ptr].modify_right(self.last_time, new_child);
                    }
                };
            }
        }

        let new_root = self.balance(self.last_time, &path);
        self.modify_root(new_root, self.last_time);

        self.last_time += 1;
        Some(self.last_time - 1)
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
