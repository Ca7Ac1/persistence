use std::cell::UnsafeCell;
use std::collections::VecDeque;

use crate::persistent_avl_tree::PersistentAvlTree;
use crate::timestamp::{get_latest, get_time, TimestampSupplier};

type FatNodePtr<'a, Data: Ord> = &'a UnsafeCell<FatNode<'a, Data>>;

fn get_fat_ptr_ref<'a, Data: Ord>(ptr: FatNodePtr<'a, Data>) -> &'a FatNode<'a, Data> {
    unsafe { &*ptr.get() }
}

fn get_fat_ptr_mut<'a, Data: Ord>(ptr: FatNodePtr<'a, Data>) -> &'a mut FatNode<'a, Data> {
    unsafe { &mut *ptr.get() }
}

#[derive(Copy, Clone, Debug)]
struct DataTime<'a, Data: Ord> {
    timestamp: u64,
    left: Option<FatNodePtr<'a, Data>>,
    right: Option<FatNodePtr<'a, Data>>,
}

impl<'a, Data: Ord> TimestampSupplier for DataTime<'a, Data> {
    type Timestamp = u64;

    fn get_timestamp(&self) -> &Self::Timestamp {
        &self.timestamp
    }
}

// Invariant: Children are in sorted order by timestamp
struct FatNode<'a, Data: Ord> {
    datum: Data,
    height: u64,
    children: Vec<DataTime<'a, Data>>,
}

impl<'a, Data: Ord> FatNode<'a, Data> {
    fn modify(&mut self, new_data: DataTime<'a, Data>) {
        self.children.push(new_data);
    }
}

struct FatNodeHead<'a, Data: Ord> {
    timestamp: u64,
    head: Option<FatNodePtr<'a, Data>>,
}

impl<'a, Data: Ord> TimestampSupplier for FatNodeHead<'a, Data> {
    type Timestamp = u64;

    fn get_timestamp(&self) -> &Self::Timestamp {
        &self.timestamp
    }
}

pub struct FatNodeAvl<'a, Data: Ord> {
    node_arena: Vec<Box<UnsafeCell<FatNode<'a, Data>>>>,
    heads: Vec<FatNodeHead<'a, Data>>,
    last_time: u64,
}

impl<'a, Data: Ord> FatNodeAvl<'a, Data> {
    fn balance_rr(
        &mut self,
        root_ptr: FatNodePtr<'a, Data>,
        timestamp: u64,
    ) -> FatNodePtr<'a, Data> {
        let root: &mut FatNode<'a, Data> = get_fat_ptr_mut(root_ptr);
        let children: &DataTime<'a, Data> =
            get_latest(&root.children).expect("Failed to find children for right rotation");

        let root_left: Option<FatNodePtr<'a, Data>> = children.left;
        let root_right: Option<FatNodePtr<'a, Data>> = children.right;

        let new_root_ptr: FatNodePtr<'a, Data> =
            root_right.expect("Failed to find right child for right rotate");

        let new_root: &mut FatNode<'a, Data> = get_fat_ptr_mut(new_root_ptr);
        let new_children =
            get_latest(&new_root.children).expect("Failed to find right child for right rotate");

        let new_root_left: Option<FatNodePtr<'a, Data>> = new_children.left;
        let new_root_right: Option<FatNodePtr<'a, Data>> = new_children.right;

        new_root.modify(DataTime {
            timestamp: timestamp,
            left: Some(root_ptr),
            right: new_root_right,
        });
        root.modify(DataTime {
            timestamp: timestamp,
            left: root_left,
            right: new_root_left,
        });

        new_root_ptr
    }

    fn balance_ll(
        &mut self,
        root_ptr: FatNodePtr<'a, Data>,
        timestamp: u64,
    ) -> FatNodePtr<'a, Data> {
        let root: &mut FatNode<'a, Data> = get_fat_ptr_mut(root_ptr);
        let children: &DataTime<'a, Data> =
            get_latest(&root.children).expect("Failed to find children for right rotation");

        let root_left: Option<FatNodePtr<'a, Data>> = children.left;
        let root_right: Option<FatNodePtr<'a, Data>> = children.right;

        let new_root_ptr: FatNodePtr<'a, Data> =
            root_left.expect("Failed to find right child for right rotate");

        let new_root: &mut FatNode<'a, Data> = get_fat_ptr_mut(new_root_ptr);
        let new_children =
            get_latest(&new_root.children).expect("Failed to find right child for right rotate");

        let new_root_left: Option<FatNodePtr<'a, Data>> = new_children.left;
        let new_root_right: Option<FatNodePtr<'a, Data>> = new_children.right;

        new_root.modify(DataTime {
            timestamp: timestamp,
            left: new_root_left,
            right: Some(root_ptr),
        });
        root.modify(DataTime {
            timestamp: timestamp,
            left: new_root_right,
            right: root_right,
        });

        new_root_ptr
    }

    fn balance_lr(
        &mut self,
        root_ptr: FatNodePtr<'a, Data>,
        timestamp: u64,
    ) -> FatNodePtr<'a, Data> {
        todo!()
    }

    fn balance_rl(
        &mut self,
        root_ptr: FatNodePtr<'a, Data>,
        timestamp: u64,
    ) -> FatNodePtr<'a, Data> {
        todo!()
    }

    fn balance(&mut self, timestamp: u64, mut prev: Vec<FatNodePtr<'a, Data>>) {
        while !prev.is_empty() {
            self.balance_one(timestamp, &mut prev);
        }
    }

    fn balance_one(&mut self, timestamp: u64, prev: &mut Vec<FatNodePtr<'a, Data>>) {
        todo!();
    }
}

impl<'a, T: Ord> PersistentAvlTree<'a> for FatNodeAvl<'a, T> {
    type Data = T;
    type Timestamp = u64;

    fn insert(&'a mut self, item: Self::Data) -> Self::Timestamp {
        self.node_arena.push(Box::new(UnsafeCell::new(FatNode {
            datum: item,
            height: 0,
            children: Vec::new(),
        })));

        let mut prev: Vec<FatNodePtr<'a, Self::Data>> = vec![];
        {
            let new_node: = &**self.node_arena.last().expect("Empty heads") as *mut UnsafeCell<FatNode<'a, T>>;

            let head_ptr = get_latest(&self.heads);

            match head_ptr.and_then(|head| head.head) {
                Some(parent_ptr) => {

                    let mut parent = get_fat_ptr_mut(parent_ptr);
                    let mut child_ptr = Some(parent_ptr);
                    while let Some(next) = child_ptr {
                        prev.push(next);
                        parent = get_fat_ptr_mut(next);

                        child_ptr = get_latest(&parent.children).and_then(|children_at_time| {
                            if parent.datum <= get_fat_ptr_ref(new_node).datum {
                                children_at_time.right
                            } else {
                                children_at_time.left
                            }
                        });
                    }

                    let children = get_latest(&parent.children);
                    if parent.datum <= get_fat_ptr_ref(new_node).datum {
                        parent.modify(DataTime {
                            timestamp: self.last_time,
                            left: children.and_then(|children_at_time| children_at_time.left),
                            right: Some(new_node),
                        });
                    } else {
                        parent.modify(DataTime {
                            timestamp: self.last_time,
                            left: Some(new_node),
                            right: children.and_then(|children_at_time| children_at_time.right),
                        });
                    }
                }
                None => {
                    self.heads.push(FatNodeHead {
                        timestamp: self.last_time,
                        head: Some(new_node),
                    });
                }
            };

        }
        
        self.balance_one(self.last_time, &mut prev);

        self.last_time += 1;
        self.last_time - 1
    }

    fn delete(&'a mut self, item: Self::Data) -> Option<Self::Timestamp> {
        todo!()
    }

    fn contains(&'a self, item: &Self::Data, timestamp: Self::Timestamp) -> bool {
        match self.predecessor(item, timestamp) {
            Some(predecessor) => *item == *predecessor,
            None => false,
        }
    }

    fn predecessor(&'a self, item: &Self::Data, timestamp: Self::Timestamp) -> Option<&Self::Data> {
        let mut root: Option<FatNodePtr<'a, T>> = get_time(&self.heads, &timestamp)?.head;
        let mut inf: Option<&Self::Data> = None;
        while let Some(node_ptr) = root {
            let node: &FatNode<'a, T> = unsafe { &*node_ptr.get() };
            let children: Option<&DataTime<'a, T>> = get_time(&node.children, &timestamp);

            match children {
                Some(children) => {
                    if node.datum > *item {
                        root = children.left;
                    } else {
                        inf = Some(&node.datum);
                        root = children.right;
                    }
                }
                None => break,
            }
        }

        inf
    }

    fn successor(&'a self, item: &Self::Data, timestamp: Self::Timestamp) -> Option<&Self::Data> {
        let mut root: Option<FatNodePtr<'a, T>> = get_time(&self.heads, &timestamp)?.head;
        let mut sup: Option<&Self::Data> = None;
        while let Some(node_ptr) = root {
            let node: &FatNode<'a, T> = unsafe { &*node_ptr.get() };
            let children: Option<&DataTime<'a, T>> = get_time(&node.children, &timestamp);

            match children {
                Some(children) => {
                    if node.datum < *item {
                        root = children.right;
                    } else {
                        sup = Some(&node.datum);
                        root = children.left;
                    }
                }
                None => break,
            }
        }

        sup
    }
}
