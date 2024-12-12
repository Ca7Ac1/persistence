use std::cmp::max;

fn set_height<NodePtr: Copy>(
    get_left: fn(NodePtr) -> Option<NodePtr>,
    get_right: fn(NodePtr) -> Option<NodePtr>,
    get_height: fn(NodePtr) -> u64,
    modify: fn(node: NodePtr, left: Option<NodePtr>, right: Option<NodePtr>, height: u64),
    node: NodePtr,
) {
    let child_height = max(
        get_left(node).map_or(0, |left_child| get_height(left_child)),
        get_right(node).map_or(0, |right_child| get_height(right_child)),
    );

    modify(node, get_left(node), get_right(node), child_height + 1);
}

fn rotate_left<NodePtr: Copy>(
    get_left: fn(NodePtr) -> Option<NodePtr>,
    get_right: fn(NodePtr) -> Option<NodePtr>,
    get_height: fn(NodePtr) -> u64,
    modify: fn(node: NodePtr, left: Option<NodePtr>, right: Option<NodePtr>, height: u64),
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

fn rotate_right<NodePtr: Copy>(
    get_left: fn(NodePtr) -> Option<NodePtr>,
    get_right: fn(NodePtr) -> Option<NodePtr>,
    get_height: fn(NodePtr) -> u64,
    modify: fn(node: NodePtr, left: Option<NodePtr>, right: Option<NodePtr>, height: u64),
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

fn get_balance_factor<NodePtr: Copy>(
    get_left: fn(NodePtr) -> Option<NodePtr>,
    get_right: fn(NodePtr) -> Option<NodePtr>,
    get_height: fn(NodePtr) -> u64,
    node: NodePtr,
) -> i32 {
    get_left(node).map_or(0, |left_child| get_height(left_child) as i32)
        - get_right(node).map_or(0, |right_child| get_height(right_child) as i32)
}

fn balance_node<NodePtr: Copy>(
    get_left: fn(NodePtr) -> Option<NodePtr>,
    get_right: fn(NodePtr) -> Option<NodePtr>,
    get_height: fn(NodePtr) -> u64,
    modify: fn(node: NodePtr, left: Option<NodePtr>, right: Option<NodePtr>, height: u64),
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

fn balance<NodePtr: Copy, Data: Ord>(
    get_left: fn(NodePtr) -> Option<NodePtr>,
    get_right: fn(NodePtr) -> Option<NodePtr>,
    get_height: fn(NodePtr) -> u64,
    get_data: fn(NodePtr) -> Data,
    modify: fn(node: NodePtr, left: Option<NodePtr>, right: Option<NodePtr>, height: u64),
    path: &Vec<NodePtr>
) -> Option<NodePtr> {
    let mut child = *path.last()?;

    path.iter().rev().skip(1).for_each(|&parent| {
        set_height(get_left, get_right, get_height, modify, child);

        child = balance_node(get_left, get_right, get_height, modify, child);

        if get_data(child) <= get_data(parent) {
            modify(
                parent,
                Some(child),
                get_right(parent),
                get_height(parent),
            );
        } else {
            modify(
                parent,
                get_left(parent),
                Some(child),
                get_height(parent),
            );
        }

        child = parent;
    });

    set_height(get_left, get_right, get_height, modify, child);
    Some(balance_node(get_left, get_right, get_height, modify, child))
}