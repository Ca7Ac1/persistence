use std::borrow::BorrowMut;
use std::cmp::max;

use crate::fat_node::*;
use crate::persistent_avl_tree::PersistentAvlTree;
use crate::timestamp::{get_latest, get_time};

pub struct FatNodeAvl<Data: Ord> {
    node_arena: Vec<Box<FatNode<Data>>>,
    root_nodes: Vec<RootNode>,
    last_time: u64,
}

impl<Data: Ord> FatNodeAvl<Data> {
    fn get_height(&self, node_ptr: Option<usize>) -> u64 {
        match node_ptr {
            Some(node_ptr) => self.node_arena[node_ptr].height,
            None => 0,
        }
    }

    fn balance_rr(&mut self, old_root_ptr: usize, timestamp: u64) -> usize {
        let old_root = self.node_arena[old_root_ptr].as_ref();

        let children =
            get_latest(&old_root.children).expect("Failed to find children for right rotation");
        let old_root_left = children.left;
        let old_root_right = children.right;

        let new_root_ptr = old_root_right.expect("Failed to find right child for right rotate");
        let new_root = self.node_arena[new_root_ptr].as_ref();

        let children = get_latest(&new_root.children)
            .expect("Failed to find right grandchildren for right rotate");
        let new_root_left = children.left;
        let new_root_right = children.right;

        let old_root_height = max(
            self.get_height(old_root_left),
            self.get_height(new_root_left),
        );
        let new_root_height = max(
            self.get_height(old_root_right),
            self.get_height(new_root_right),
        );

        self.node_arena[old_root_ptr].modify_right(timestamp, old_root_height, new_root_left);
        self.node_arena[new_root_ptr].modify_left(timestamp, new_root_height, old_root_right);

        new_root_ptr
    }
}

impl<Data: Ord> PersistentAvlTree for FatNodeAvl<Data> {
    type Data = Data;
    type Timestamp = u64;
    
    fn insert(&mut self, item: Self::Data) -> Self::Timestamp {
        todo!()
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
            let node = self.node_arena[current_ptr].as_ref();
            let children= get_time(&node.children, &timestamp);

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
            let node = self.node_arena[current_ptr].as_ref();
            let children= get_time(&node.children, &timestamp);

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