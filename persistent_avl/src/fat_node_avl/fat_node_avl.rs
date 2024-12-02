use std::cmp::max;

use crate::fat_node_avl::fat_node::{FatNode, RootNode};
use crate::persistent_avl_tree::PersistentAvlTree;
use crate::timestamp::{self, get_time};

pub struct FatNodeAvl<Data: Ord> {
    node_arena: Vec<FatNode<Data>>,
    root_nodes: Vec<RootNode>,
    last_time: u64,
}

impl<Data: Ord> FatNodeAvl<Data> {
    fn modify_root(&mut self, new_node_ptr: Option<usize>, timestamp: u64) {
        match self
            .root_nodes
            .last_mut()
            .filter(|last_root| last_root.timestamp == timestamp)
        {
            Some(same_root) => same_root.root = new_node_ptr,
            None => self.root_nodes.push(RootNode {
                timestamp,
                root: new_node_ptr,
            }),
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
        !todo!()
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
