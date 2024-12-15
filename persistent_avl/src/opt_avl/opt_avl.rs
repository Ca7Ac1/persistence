use std::collections::BTreeMap;

use super::opt::OptAVLNode;

pub struct OptAVL<'a, Data: Ord, Timestamp: Ord> {
    node_arena: Vec<OptAVLNode<'a, Timestamp>>,
    data_arena: Vec<Data>,
    roots: BTreeMap<&'a Timestamp, Option<usize>>
}

enum ParentToChildDirection {
    Left,
    Right,
    RootToChild,
}

impl<'a, Data: Ord, Timestamp: Ord> OptAVL<'a, Data, Timestamp> {
    pub(crate) fn get_left(&self, node_ptr: Option<usize>, timestamp: &Timestamp) -> Option<usize> {
        node_ptr.and_then(|node_ptr| self.node_arena[node_ptr].get_left(timestamp))
    }

    pub(crate) fn get_right(&self, node_ptr: Option<usize>, timestamp: &Timestamp) -> Option<usize> {
        node_ptr.and_then(|node_ptr| self.node_arena[node_ptr].get_right(timestamp))
    }

    /// Precondition: the given node is a L2 or R2 child of the last ancestor, or is the root.
    fn get_parent_direction(&self, ancestors: &[usize], node: usize, timestamp: &Timestamp) -> ParentToChildDirection {
        if ancestors.is_empty() {
            return ParentToChildDirection::RootToChild;
        }
        let parent_ptr = ancestors.last().unwrap();
        if self.get_left(Some(*parent_ptr), timestamp) == Some(node) {
            ParentToChildDirection::Left
        } else {
            ParentToChildDirection::Right
        }
    }

    /// Updates the last ancestor's l2 := node_ptr.
    /// If this operation causes the parent to duplicate, also
    /// calls update_xx_pointer(ancestors[..|n - 1|], parent_duplicate)
    /// 
    /// Precondition: timestamp is newest
    /// Precondition: all ancestors are L2/R2 children
    pub(crate) fn update_left_pointer(&mut self, ancestors: &[usize], node_ptr: Option<usize>, timestamp: &'a Timestamp) {
        if ancestors.is_empty() {
            self.roots.insert(timestamp, node_ptr);
            return;
        }

        let parent = ancestors.last().unwrap();
        let result = self.node_arena[*parent].modify_left_or_duplicate(node_ptr, timestamp);
        
        if let Some(duplicate) = result {
            // Allocate
            self.node_arena.push(duplicate);
            let duplicate_ptr = self.node_arena.len() - 1;

            // Check the direction of the grandparent to the parent
            let gpp_dir = self.get_parent_direction(ancestors, *parent, timestamp);
            match gpp_dir {
                ParentToChildDirection::Left => self.update_left_pointer(&ancestors[0..ancestors.len() - 1], Some(duplicate_ptr), timestamp),
                ParentToChildDirection::Right => self.update_right_pointer(&ancestors[0..ancestors.len() - 1], Some(duplicate_ptr), timestamp),
                ParentToChildDirection::RootToChild => { self.roots.insert(timestamp, node_ptr); },
            }
        }
    }

    /// Updates the last ancestor's r2 := node_ptr.
    /// If this operation causes the parent to duplicate, also
    /// calls update_xx_pointer(ancestors[..|n - 1|], parent_duplicate)
    /// 
    /// Precondition: timestamp is newest
    /// Precondition: all ancestors are L2/R2 children
    pub(crate) fn update_right_pointer(&mut self, ancestors: &[usize], node_ptr: Option<usize>, timestamp: &'a Timestamp) {
        if ancestors.is_empty() {
            self.roots.insert(timestamp, node_ptr);
            return;
        }

        let parent = ancestors.last().unwrap();
        let result = self.node_arena[*parent].modify_right_or_duplicate(node_ptr, timestamp);
        
        if let Some(duplicate) = result {
            // Allocate
            self.node_arena.push(duplicate);
            let duplicate_ptr = self.node_arena.len() - 1;

            // Check the direction of the grandparent to the parent
            let gpp_dir = self.get_parent_direction(ancestors, *parent, timestamp);
            match gpp_dir {
                ParentToChildDirection::Left => self.update_left_pointer(&ancestors[0..ancestors.len() - 1], Some(duplicate_ptr), timestamp),
                ParentToChildDirection::Right => self.update_right_pointer(&ancestors[0..ancestors.len() - 1], Some(duplicate_ptr), timestamp),
                ParentToChildDirection::RootToChild => { self.roots.insert(timestamp, node_ptr); },
            }
        }
    }

    /// Precondition: timestamp is newest
    pub fn insert(&mut self, datum: Data, timestamp: &'a Timestamp) {
        // Allocate datum
        self.data_arena.push(datum);
        let datum_ptr = self.data_arena.len() - 1;
        let datum = &self.data_arena[datum_ptr];

        // Allocate node
        self.node_arena.push(OptAVLNode {
            datum_ptr,
            height: 0,
            timestamp,
            l1: None,
            r1: None,
            l2: None,
            r2: None,
        });
        let node_ptr = self.node_arena.len() - 1;

        // Traverse
        let mut path_ptr = self.roots.last_key_value().and_then(|(_, ptr)| *ptr);

        let mut path = Vec::new();
        let mut last_dir = ParentToChildDirection::RootToChild;
        while let Some(ptr) = path_ptr {
            path.push(ptr);

            let node_datum = &self.data_arena[self.node_arena[ptr].datum_ptr];

            if *datum <= *node_datum {
                path_ptr = self.get_left(Some(ptr), timestamp);
                last_dir = ParentToChildDirection::Left;
            } else {
                path_ptr = self.get_right(Some(ptr), timestamp);
                last_dir = ParentToChildDirection::Right;
            }
        }

        match last_dir {
            ParentToChildDirection::Left => self.update_left_pointer(&path, Some(node_ptr), timestamp),
            ParentToChildDirection::Right => self.update_right_pointer(&path, Some(node_ptr), timestamp),
            ParentToChildDirection::RootToChild => { self.roots.insert(timestamp, Some(node_ptr)); },
        }
    }
}
