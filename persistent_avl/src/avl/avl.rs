use std::cmp::{max, Ordering};

pub(crate) fn set_height<NodePtr: Copy>(
    get_left: &impl Fn(NodePtr) -> Option<NodePtr>,
    get_right: &impl Fn(NodePtr) -> Option<NodePtr>,
    get_height: &impl Fn(NodePtr) -> u64,
    modify: &mut impl FnMut(NodePtr, Option<NodePtr>, Option<NodePtr>, u64),
    node: NodePtr,
) {
    let child_height = max(
        get_left(node).map_or(0, |left_child| get_height(left_child)),
        get_right(node).map_or(0, |right_child| get_height(right_child)),
    );

    modify(node, get_left(node), get_right(node), child_height + 1);
}

pub(crate) fn rotate_left<NodePtr: Copy>(
    get_left: &impl Fn(NodePtr) -> Option<NodePtr>,
    get_right: &impl Fn(NodePtr) -> Option<NodePtr>,
    get_height: &impl Fn(NodePtr) -> u64,
    modify: &mut impl FnMut(NodePtr, Option<NodePtr>, Option<NodePtr>, u64),
    original_root: NodePtr,
) -> NodePtr {
    let new_root = get_right(original_root).unwrap();

    let original_root_left = get_left(original_root);
    let original_root_right = get_left(new_root);

    let new_root_left = Some(original_root);
    let new_root_right = get_right(new_root);

    // Input node is now lower
    modify(
        original_root,
        original_root_left,
        original_root_right,
        get_height(original_root),
    );

    // Child node is now upper
    modify(
        new_root,
        new_root_left,
        new_root_right,
        get_height(new_root),
    );

    set_height(get_left, get_right, get_height, modify, original_root);
    set_height(get_left, get_right, get_height, modify, new_root);

    new_root
}

pub(crate) fn rotate_right<NodePtr: Copy>(
    get_left: &impl Fn(NodePtr) -> Option<NodePtr>,
    get_right: &impl Fn(NodePtr) -> Option<NodePtr>,
    get_height: &impl Fn(NodePtr) -> u64,
    modify: &mut impl FnMut(NodePtr, Option<NodePtr>, Option<NodePtr>, u64),
    original_root: NodePtr,
) -> NodePtr {
    let new_root = get_left(original_root).unwrap();

    let original_root_left = get_right(new_root);
    let original_root_right = get_right(original_root);

    let new_root_left = get_left(new_root);
    let new_root_right = Some(original_root);

    // Input node is now lower
    modify(
        original_root,
        original_root_left,
        original_root_right,
        get_height(original_root),
    );

    // Child node is now upper
    modify(
        new_root,
        new_root_left,
        new_root_right,
        get_height(new_root),
    );

    set_height(get_left, get_right, get_height, modify, original_root);
    set_height(get_left, get_right, get_height, modify, new_root);

    new_root
}

pub(crate) fn get_balance_factor<NodePtr: Copy>(
    get_left: &impl Fn(NodePtr) -> Option<NodePtr>,
    get_right: &impl Fn(NodePtr) -> Option<NodePtr>,
    get_height: &impl Fn(NodePtr) -> u64,
    node: NodePtr,
) -> i32 {
    get_left(node).map_or(0, |left_child| get_height(left_child) as i32)
        - get_right(node).map_or(0, |right_child| get_height(right_child) as i32)
}

pub(crate) fn balance_node<NodePtr: Copy>(
    get_left: &impl Fn(NodePtr) -> Option<NodePtr>,
    get_right: &impl Fn(NodePtr) -> Option<NodePtr>,
    get_height: &impl Fn(NodePtr) -> u64,
    modify: &mut impl FnMut(NodePtr, Option<NodePtr>, Option<NodePtr>, u64),
    node: NodePtr,
) -> NodePtr {
    let balance = get_balance_factor(get_left, get_right, get_height, node);

    if balance <= -2 {
        let left_child = get_left(node).unwrap();

        // LR
        if get_balance_factor(get_left, get_right, get_height, left_child) >= 1 {
            let new_left_child = rotate_left(get_left, get_right, get_height, modify, left_child);

            modify(
                node,
                Some(new_left_child),
                get_right(node),
                get_height(node),
            );
        }

        // LL & LR
        rotate_right(get_left, get_right, get_height, modify, node)
    } else if balance >= 2 {
        let right_child = get_right(node).unwrap();

        // RL
        if get_balance_factor(get_left, get_right, get_height, right_child) <= -1 {
            let new_right_child =
                rotate_right(get_left, get_right, get_height, modify, right_child);

            modify(
                node,
                get_left(node),
                Some(new_right_child),
                get_height(node),
            );
        }

        // RL & RR
        rotate_left(get_left, get_right, get_height, modify, node)
    } else {
        node
    }
}

pub(crate) fn balance<NodePtr: Copy>(
    get_left: &impl Fn(NodePtr) -> Option<NodePtr>,
    get_right: &impl Fn(NodePtr) -> Option<NodePtr>,
    get_height: &impl Fn(NodePtr) -> u64,
    compare: &impl Fn(NodePtr, NodePtr) -> Ordering,
    modify: &mut impl FnMut(NodePtr, Option<NodePtr>, Option<NodePtr>, u64),
    path: &Vec<NodePtr>,
) -> Option<NodePtr> {
    let mut child = *path.last()?;

    path.iter().rev().skip(1).for_each(|&parent| {
        set_height(get_left, get_right, get_height, modify, child);

        child = balance_node(get_left, get_right, get_height, modify, child);

        match compare(child, parent) {
            Ordering::Less | Ordering::Equal => {
                modify(parent, Some(child), get_right(parent), get_height(parent));
            }
            Ordering::Greater => {
                modify(parent, get_left(parent), Some(child), get_height(parent));
            }
        }

        child = parent;
    });

    set_height(get_left, get_right, get_height, modify, child);
    Some(balance_node(get_left, get_right, get_height, modify, child))
}

// pub(crate) fn insert(&mut self, item: Self::Data) -> Self::Timestamp {
//     // Allocation
//     self.node_arena.push(FatNode {
//         datum: item,
//         height: 1,
//         children: Vec::new(),
//     });
//     let new_node_ptr = self.node_arena.len() - 1;
//     let item = &self.node_arena[new_node_ptr].datum;

//     // Insertion
//     let root = self.root_nodes.last().and_then(|root_node| root_node.root);

//     match root {
//         Some(mut parent_ptr) => {
//             let mut path = vec![parent_ptr];

//             while let Some(child_ptr) =
//                 self.node_arena[parent_ptr]
//                     .children
//                     .last()
//                     .and_then(|children| {
//                         if *item <= self.node_arena[parent_ptr].datum {
//                             children.left
//                         } else {
//                             children.right
//                         }
//                     })
//             {
//                 path.push(child_ptr);
//                 parent_ptr = child_ptr;
//             }

//             if *item <= self.node_arena[parent_ptr].datum {
//                 self.node_arena[parent_ptr].modify_left(self.last_time, Some(new_node_ptr));
//             } else {
//                 self.node_arena[parent_ptr].modify_right(self.last_time, Some(new_node_ptr));
//             }

//             let new_root = self.balance(self.last_time, &path);
//             self.modify_root(new_root, self.last_time);
//         }
//         None => self.modify_root(Some(new_node_ptr), self.last_time),
//     }

//     self.last_time += 1;
//     self.last_time - 1
// }

// pub(crate) fn delete(&mut self, item: &Self::Data) -> Option<Self::Timestamp> {
//     let mut parent_ptr = None;
//     let mut child_ptr = self
//         .root_nodes
//         .last()
//         .and_then(|root_node| root_node.root)?;

//     // Path keeping track of all modified nodes in order
//     let mut path = Vec::new();

//     // Traverse to node to delete
//     while self.node_arena[child_ptr].datum != *item {
//         path.push(child_ptr);
//         parent_ptr = Some(child_ptr);

//         let children = self.node_arena[child_ptr].children.last()?;

//         child_ptr = if self.node_arena[child_ptr].datum < *item {
//             children.right?
//         } else {
//             children.left?
//         };
//     }

//     let children_of_deleted = self.node_arena[child_ptr].children.last();

//     let left_of_deleted = children_of_deleted.and_then(|children| children.left);
//     let right_of_deleted = children_of_deleted.and_then(|children| children.left);

//     // match against both children of deleted node existing
//     match left_of_deleted.zip(right_of_deleted) {
//         Some((_, right_subtree_ptr)) => {
//             match self.node_arena[right_subtree_ptr]
//                 .children
//                 .last()
//                 .and_then(|children| children.left)
//             {
//                 // If the left child of our right subtree exists we find the
//                 // successor of our deleted node. Note that this successor
//                 // necessarily doesn't have a right child, as otherwise that
//                 // would be our successor. We replace our deleted node with its
//                 // successor, and give the successors right child to its parent.
//                 Some(mut sup_ptr) => {
//                     let mut sup_parent_ptr = right_subtree_ptr;
//                     let mut displaced_path = vec![sup_parent_ptr];

//                     while let Some(lesser) = self.node_arena[sup_ptr]
//                         .children
//                         .last()
//                         .and_then(|children| children.left)
//                     {
//                         sup_parent_ptr = sup_ptr;
//                         displaced_path.push(sup_parent_ptr);

//                         sup_ptr = lesser;
//                     }

//                     // Our path will be up to the deleted node, then the next
//                     // node will be our successor, and then the next nodes will
//                     // be the the path down to where our successor was located.
//                     path.push(sup_ptr);
//                     path.append(&mut displaced_path);

//                     let right_of_sup = self.node_arena[sup_ptr]
//                         .children
//                         .last()
//                         .and_then(|children| children.right);
//                     self.node_arena[sup_parent_ptr].modify_left(self.last_time, right_of_sup);

//                     self.node_arena[sup_ptr].modify_left(self.last_time, left_of_deleted);
//                     self.node_arena[sup_ptr].modify_right(self.last_time, right_of_deleted);

//                     if let Some(parent_ptr) = parent_ptr {
//                         if self.node_arena[parent_ptr].children.last().unwrap().left
//                             == Some(child_ptr)
//                         {
//                             self.node_arena[parent_ptr]
//                                 .modify_left(self.last_time, Some(sup_ptr));
//                         } else {
//                             self.node_arena[parent_ptr]
//                                 .modify_right(self.last_time, Some(sup_ptr));
//                         }
//                     };
//                 },

//                 // If the left child of our right subtree does not exist,
//                 // we give the left child of our deleted node to the right
//                 // subtree, and replace our deleted node with its right child.
//                 None => {
//                     self.node_arena[right_subtree_ptr]
//                         .modify_left(self.last_time, left_of_deleted);

//                     if let Some(parent_ptr) = parent_ptr {
//                         if self.node_arena[parent_ptr].children.last().unwrap().left
//                             == Some(child_ptr)
//                         {
//                             self.node_arena[parent_ptr]
//                                 .modify_left(self.last_time, Some(right_subtree_ptr));
//                         } else {
//                             self.node_arena[parent_ptr]
//                                 .modify_right(self.last_time, Some(right_subtree_ptr));
//                         }
//                     };
//                 }
//             }
//         }
//         // If the deleted node has a single child then we replace it with that child.
//         // Otherwise if the deleted node has no children we remove it without replacement.
//         None => {
//             let new_child = if let None = left_of_deleted {
//                 right_of_deleted
//             } else if let None = right_of_deleted {
//                 left_of_deleted
//             } else {
//                 None
//             };

//             if let Some(parent_ptr) = parent_ptr {
//                 if self.node_arena[parent_ptr].children.last().unwrap().left == Some(child_ptr)
//                 {
//                     self.node_arena[parent_ptr].modify_left(self.last_time, new_child);
//                 } else {
//                     self.node_arena[parent_ptr].modify_right(self.last_time, new_child);
//                 }
//             };
//         }
//     }

//     let new_root = self.balance(self.last_time, &path);
//     self.modify_root(new_root, self.last_time);

//     self.last_time += 1;
//     Some(self.last_time - 1)
// }

// pub(crate) fn contains(&self, item: &Self::Data, timestamp: Self::Timestamp) -> bool {
//     match self.predecessor(item, timestamp) {
//         Some(predecessor) => *item == *predecessor,
//         None => false,
//     }
// }

pub(crate) fn contains<NodePtr: Copy, Data>(
    get_left: &impl Fn(NodePtr) -> Option<NodePtr>,
    get_right: &impl Fn(NodePtr) -> Option<NodePtr>,
    compare: &impl Fn(&Data, NodePtr) -> Ordering,
    root: Option<NodePtr>,
    data: &Data,
) -> bool {
    predecessor(get_left, get_right, compare, root, data)
        .map_or(false, |node| compare(data, node) == Ordering::Equal)
}

pub(crate) fn predecessor<NodePtr: Copy, Data>(
    get_left: &impl Fn(NodePtr) -> Option<NodePtr>,
    get_right: &impl Fn(NodePtr) -> Option<NodePtr>,
    compare: &impl Fn(&Data, NodePtr) -> Ordering,
    mut root: Option<NodePtr>,
    data: &Data,
) -> Option<NodePtr> {
    let mut inf: Option<NodePtr> = None;

    while let Some(current) = root {
        match compare(data, current) {
            Ordering::Less => {
                root = get_left(current);
            }
            Ordering::Greater | Ordering::Equal => {
                inf = Some(current);
                root = get_right(current);
            }
        };
    }

    inf
}

pub(crate) fn successor<NodePtr: Copy, Data>(
    get_left: &impl Fn(NodePtr) -> Option<NodePtr>,
    get_right: &impl Fn(NodePtr) -> Option<NodePtr>,
    compare: &impl Fn(&Data, NodePtr) -> Ordering,
    mut root: Option<NodePtr>,
    data: &Data,
) -> Option<NodePtr> {
    let mut sup: Option<NodePtr> = None;

    while let Some(current) = root {
        match compare(data, current) {
            Ordering::Less => {
                root = get_right(current);
            }
            Ordering::Greater | Ordering::Equal => {
                sup = Some(current);
                root = get_left(current);
            }
        };
    }

    sup
}
