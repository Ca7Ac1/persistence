use std::cmp::max;

fn set_height<Tree, NodePtr: Copy>(
    tree: &mut Tree, 
    get_left: fn(&Tree, NodePtr) -> Option<NodePtr>,
    get_right: fn(&Tree, NodePtr) -> Option<NodePtr>,
    get_height: fn(&Tree, NodePtr) -> u64,
    modify: fn(tree: &mut Tree, node: NodePtr, left: Option<NodePtr>, right: Option<NodePtr>, height: u64),
    node: NodePtr,
) {
    let child_height = max(
        get_left(tree, node).map_or(0, |left_child| get_height(tree, left_child)),
        get_right(tree, node).map_or(0, |right_child| get_height(tree, right_child)),
    );

    modify(tree, node, get_left(tree, node), get_right(tree, node), child_height + 1);
}

fn rotate_left<Tree, NodePtr: Copy>(
    tree: &mut Tree, 
    get_left: fn(&Tree, NodePtr) -> Option<NodePtr>,
    get_right: fn(&Tree, NodePtr) -> Option<NodePtr>,
    get_height: fn(&Tree, NodePtr) -> u64,
    modify: fn(tree: &mut Tree, node: NodePtr, left: Option<NodePtr>, right: Option<NodePtr>, height: u64),
    original_root: NodePtr,
) -> NodePtr {
    let new_root = get_right(tree, original_root).unwrap();

    let original_root_left = get_left(tree, original_root);
    let original_root_right = get_left(tree, new_root);

    let new_root_left = Some(original_root);
    let new_root_right = get_right(tree, new_root);

    // Input node is now lower
    modify(
        tree,
        original_root,
        original_root_left,
        original_root_right,
        get_height(tree, original_root),
    );

    // Child node is now upper
    modify(
        tree,
        new_root,
        new_root_left,
        new_root_right,
        get_height(tree, new_root),
    );

    set_height(tree,get_left, get_right, get_height, modify, original_root);
    set_height(tree, get_left, get_right, get_height, modify, new_root);

    new_root
}

fn rotate_right<Tree, NodePtr: Copy>(
    tree: &mut Tree, 
    get_left: fn(&Tree, NodePtr) -> Option<NodePtr>,
    get_right: fn(&Tree, NodePtr) -> Option<NodePtr>,
    get_height: fn(&Tree, NodePtr) -> u64,
    modify: fn(tree: &mut Tree, node: NodePtr, left: Option<NodePtr>, right: Option<NodePtr>, height: u64),
    original_root: NodePtr,
) -> NodePtr {
    let new_root = get_left(tree, original_root).unwrap();

    let original_root_left = get_right(tree, new_root);
    let original_root_right = get_right(tree, original_root);

    let new_root_left = get_left(tree, new_root);
    let new_root_right = Some(original_root);

    // Input node is now lower
    modify(
        tree,
        original_root,
        original_root_left,
        original_root_right,
        get_height(tree, original_root),
    );

    // Child node is now upper
    modify(
        tree,
        new_root,
        new_root_left,
        new_root_right,
        get_height(tree, new_root),
    );

    set_height(tree, get_left, get_right, get_height, modify, original_root);
    set_height(tree, get_left, get_right, get_height, modify, new_root);

    new_root
}

fn get_balance_factor<Tree, NodePtr: Copy>(
    tree: &mut Tree, 
    get_left: fn(&Tree, NodePtr) -> Option<NodePtr>,
    get_right: fn(&Tree, NodePtr) -> Option<NodePtr>,
    get_height: fn(&Tree, NodePtr) -> u64,
    node: NodePtr,
) -> i32 {
    get_left(tree, node).map_or(0, |left_child| get_height(tree, left_child) as i32)
        - get_right(tree, node).map_or(0, |right_child| get_height(tree, right_child) as i32)
}

fn balance_node<Tree, NodePtr: Copy>(
    tree: &mut Tree, 
    get_left: fn(&Tree, NodePtr) -> Option<NodePtr>,
    get_right: fn(&Tree, NodePtr) -> Option<NodePtr>,
    get_height: fn(&Tree, NodePtr) -> u64,
    modify: fn(tree: &mut Tree, node: NodePtr, left: Option<NodePtr>, right: Option<NodePtr>, height: u64),
    node: NodePtr,
) -> NodePtr {
    let balance = get_balance_factor(tree, get_left, get_right, get_height, node);

    if balance <= -2 {
        let left_child = get_left(tree, node).unwrap();

        // LR
        if get_balance_factor(tree, get_left, get_right, get_height, left_child) >= 1 {
            let new_left_child = rotate_left(tree, get_left, get_right, get_height, modify, left_child);

            modify(
                tree,
                node,
                Some(new_left_child),
                get_right(tree, node),
                get_height(tree, node),
            );
        }

        // LL & LR
        rotate_right(tree, get_left, get_right, get_height, modify, node)
    } else if balance >= 2 {
        let right_child = get_right(tree, node).unwrap();

        // RL
        if get_balance_factor(tree, get_left, get_right, get_height, right_child) <= -1 {
            let new_right_child =
                rotate_right(tree, get_left, get_right, get_height, modify, right_child);

            modify(
                tree, 
                node,
                get_left(tree, node),
                Some(new_right_child),
                get_height(tree, node),
            );
        }

        // RL & RR
        rotate_left(tree, get_left, get_right, get_height, modify, node)
    } else {
        node
    }
}

fn balance<Tree, NodePtr: Copy, Data: Ord>(
    tree: &mut Tree, 
    get_left: fn(&Tree, NodePtr) -> Option<NodePtr>,
    get_right: fn(&Tree, NodePtr) -> Option<NodePtr>,
    get_height: fn(&Tree, NodePtr) -> u64,
    get_data: fn(&Tree, NodePtr) -> Data,
    modify: fn(tree: &mut Tree, node: NodePtr, left: Option<NodePtr>, right: Option<NodePtr>, height: u64),
    path: &Vec<NodePtr>
) -> Option<NodePtr> {
    let mut child = *path.last()?;

    path.iter().rev().skip(1).for_each(|&parent| {
        set_height(tree, get_left, get_right, get_height, modify, child);

        child = balance_node(tree, get_left, get_right, get_height, modify, child);

        if get_data(tree, child) <= get_data(tree, parent) {
            modify(
                tree,
                parent,
                Some(child),
                get_right(tree, parent),
                get_height(tree, parent),
            );
        } else {
            modify(
                tree,
                parent,
                get_left(tree, parent),
                Some(child),
                get_height(tree, parent),
            );
        }

        child = parent;
    });

    set_height(tree, get_left, get_right, get_height, modify, child);
    Some(balance_node(tree, get_left, get_right, get_height, modify, child))
}