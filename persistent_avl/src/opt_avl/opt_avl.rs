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
}
