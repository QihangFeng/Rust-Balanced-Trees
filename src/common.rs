use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::{Rc, Weak};

#[derive(Debug)]
pub enum InsertionPoint<NRef> {
    // Tree is empty -> new node becomes root.
    AsRoot,
    // Key already exists at this node.
    Existing(NRef),
    // Insert as the left child of parent.
    AtLeftOf(NRef),
    // Insert as the right child of parent.
    AtRightOf(NRef),
}

#[derive(Debug, PartialEq, Eq)]
pub enum LocationRel {
    LL,
    RR,
    LR,
    RL,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayColor {
    Red,
    Black,
}

pub trait TreeNode<T> {
    // read-only
    fn value(&self) -> &T;
    fn left(&self) -> &Option<Rc<RefCell<Self>>>;
    fn right(&self) -> &Option<Rc<RefCell<Self>>>;
    fn parent(&self) -> &Option<Weak<RefCell<Self>>>;
    // Default color is None. Only RBNode will overwrite color.
    fn color(&self) -> Option<DisplayColor> {
        None
    }
    // AVL have its height
    fn height(&self) -> Option<usize> {
        None
    }
    
    fn to_string(&self) -> String {
        "Node.to_string() is not implemented.".to_string()
    }
    // mutable access for rotation
    fn left_mut(&mut self) -> &mut Option<Rc<RefCell<Self>>>;
    fn right_mut(&mut self) -> &mut Option<Rc<RefCell<Self>>>;
    fn parent_mut(&mut self) -> &mut Option<Weak<RefCell<Self>>>;
}

pub trait TreeOps<T: Ord + Copy + Debug> {
    fn insert(&mut self, v: Vec<T>);
    fn delete(&mut self, v: Vec<T>);
    fn count_leaves(&self) -> usize;
    fn height(&self) -> usize;
    fn inorder_traversal(&self);
    fn is_tree_empty(&self) -> bool;
    fn print_tree_pattern1(&self);
    fn print_tree_pattern2(&self);
    fn clear(&mut self);
}

// Signatures of ops 3,4,5,7
pub fn count_leaves<T, N>(node: &Option<Rc<RefCell<N>>>) -> usize
where
    N: TreeNode<T>,
{
    if let Some(n) = node {
        if n.borrow().left().is_none() && n.borrow().right().is_none() {
            1
        } else {
            count_leaves(&n.borrow().left()) + count_leaves(&n.borrow().right())
        }
    } else {
        0
    }
}

pub fn height<T, N>(node: &Option<Rc<RefCell<N>>>) -> usize
where
    N: TreeNode<T>,
{
    if let Some(rc) = node {
        let left_h = height(&rc.borrow().left());
        let right_h = height(&rc.borrow().right());
        let h = left_h.max(right_h) + 1;
        h
    } else {
        0
    }
}

pub fn inorder_traversal<T, N>(node: &Option<Rc<RefCell<N>>>)
where
    T: Debug,
    N: TreeNode<T>,
{
    if let Some(n) = node {
        inorder_traversal(&n.borrow().left());
        print!("{:?} ", n.borrow().value());
        inorder_traversal(&n.borrow().right());
    }
}

pub fn print_tree_pattern1<T, N>(node: &Option<Rc<RefCell<N>>>)
where
    T: Ord + Copy + Debug,
    N: TreeNode<T>,
{
    fn print_layer<T, N>(node: &Option<Rc<RefCell<N>>>, prefix: &str, is_left: bool, is_last: bool)
    where
        T: Ord + Copy + Debug,
        N: TreeNode<T>,
    {
        // node is an Option, `if let` control the recursion end.
        // Some -> continue recursion; None -> recursion stops
        if let Some(rc) = node {
            let n_ref = rc.borrow();
            // connector before the node value
            let connector = if prefix.is_empty() {
                "Root: " // root
            } else if is_left {
                if is_last {
                    "└── L: "
                } else {
                    "├── L: "
                }
            } else {
                "└── R: "
            };

            let (color_start, color_end, tag): (&str, &str, String) = match n_ref.color() {
                Some(DisplayColor::Red) => ("\x1b[31m", "\x1b[0m", " [red]".to_string()), // red
                Some(DisplayColor::Black) => ("\x1b[90m", "\x1b[0m", " [black]".to_string()), // dark grey
                None => ("", "", format!(" [height: {}]", n_ref.height().unwrap())),
            };
            println!(
                "{}{}{}{:?}{}{}",
                prefix,
                connector,
                color_start,
                n_ref.value(),
                tag,
                color_end
            );
            // build prefix for children to control the indention
            let child_prefix = if prefix.is_empty() {
                "  ".to_string()
            } else if is_last {
                format!("{prefix}    ")
            } else {
                format!("{prefix}│   ")
            };

            let has_left = n_ref.left().is_some();
            let has_right = n_ref.right().is_some();
            if has_left {
                // left is last only if there is NO right child
                let left_is_last = !has_right;
                print_layer::<T, N>(n_ref.left(), &child_prefix, true, left_is_last);
            }
            if has_right {
                // right child is always the last
                print_layer::<T, N>(n_ref.right(), &child_prefix, false, true);
            }
        }
    }
    if let Some(rc) = node {
        print_layer::<T, N>(node, "", false, true);
    } else {
        println!("Empty Tree")
    }
}


pub fn print_tree_pattern2<T, N>(node: &Option<Rc<RefCell<N>>>)
where
    T: Ord + Copy + Debug,
    N: TreeNode<T>,
{
    fn print_layer<T, N>(
        node: &Option<Rc<RefCell<N>>>,
        prefix: &String,
        is_left: bool
    ) where
        T: Ord + Copy + Debug,
        N: TreeNode<T>,
    {
        {
            if let Some(rc) = node {
                let n_ref = rc.borrow();

                let has_right = n_ref.right().is_some();

                if has_right {
                    let right_prefix = if prefix.is_empty() {
                        format!("  │")
                    } else if is_left {
                        format!("{prefix}   │")
                    } else {
                        format!("{prefix}\x08    │")
                    };
                    // right child is always the last sibling
                    print_layer::<T, N>(n_ref.right(), &right_prefix, false);
                    println!("{}", right_prefix);
                }

                // connector before the node value
                let connector = if prefix.is_empty() {
                    "Root: " // root
                } else if is_left {
                    "\x08└── L: "
                } else {
                    "\x08┌── R: "
                };

                println!(
                    "{}{}{}",
                    prefix,
                    connector,
                    n_ref.to_string()
                );

                let has_left = n_ref.left().is_some();

                if has_left {
                    let left_prefix = if prefix.is_empty() {
                        format!("  │")
                    } else if is_left {
                        format!("{prefix}\x08    │")
                    } else {
                        format!("{prefix}   │")
                    };
                    println!("{}", left_prefix);
                    print_layer::<T, N>(n_ref.left(), &left_prefix, true);
                }
            }
        }
    }

    if let Some(_) = node {
        print_layer::<T, N>(node, &"".to_string(), false);
    } else {
        println!("Empty Tree");
    }
}


// Shared helpers
pub fn find_location<T, N>(root: &Option<Rc<RefCell<N>>>, key: T) -> InsertionPoint<Rc<RefCell<N>>>
where
    T: Ord + Copy + Debug,
    N: TreeNode<T>,
{
    let mut parent: Option<Rc<RefCell<N>>> = None;
    let mut cur = root.clone();

    while let Some(node) = cur {
        let v = *node.borrow().value();
        match key.cmp(&v) {
            std::cmp::Ordering::Equal => {
                return InsertionPoint::Existing(node.clone());
            }
            std::cmp::Ordering::Less => {
                parent = Some(node.clone());
                cur = node.borrow().left().clone();
            }
            std::cmp::Ordering::Greater => {
                parent = Some(node.clone());
                cur = node.borrow().right().clone();
            }
        }
    }

    match parent {
        None => InsertionPoint::AsRoot,
        Some(p) => {
            if key < *p.borrow().value() {
                InsertionPoint::AtLeftOf(p)
            } else {
                InsertionPoint::AtRightOf(p)
            }
        }
    }
}

pub fn find_location_rel<T, N>(node: &Rc<RefCell<N>>) -> Option<LocationRel>
where
    T: Ord + Copy,
    N: TreeNode<T>,
{
    let parent = get_parent::<T, N>(node)?;
    let grandparent = get_grandparent::<T, N>(node)?;
    let parent_is_left = is_left_child::<T, N>(&parent);
    let node_is_left = is_left_child::<T, N>(node);

    Some(match (parent_is_left, node_is_left) {
        (true, true) => LocationRel::LL,
        (false, false) => LocationRel::RR,
        (true, false) => LocationRel::LR,
        (false, true) => LocationRel::RL,
    })
}

pub fn rotate<T, N>(root: &mut Option<Rc<RefCell<N>>>, node: &Rc<RefCell<N>>, loc_rel: LocationRel)
where
    T: Ord + Copy,
    N: TreeNode<T>,
{
    let parent = get_parent(node).expect("rotate: parent doesn't exist!");
    let grandparent = get_grandparent(node).expect("rotate: grandparent doesn't exist!");
    match loc_rel {
        LocationRel::LL => {
            // parent_right = parent.right, parent.right = None
            let parent_right = { parent.borrow_mut().right_mut().take() };
            {
                // grandparent.left = parent_right
                *(grandparent.borrow_mut().left_mut()) = parent_right.clone();
            }
            if let Some(ref pr) = parent_right {
                // parent_right.parent = grandparent
                *(pr.borrow_mut().parent_mut()) = Some(Rc::downgrade(&grandparent));
            }
            if let Some(grandparent_parent) = get_parent(&grandparent) {
                if is_left_child(&grandparent) {
                    // grandparent_parent.(left|right) = parent
                    *(grandparent_parent.borrow_mut().left_mut()) = Some(parent.clone());
                } else {
                    *(grandparent_parent.borrow_mut().right_mut()) = Some(parent.clone());
                }
                // parent.parent = grandparent
                *(parent.borrow_mut().parent_mut()) = Some(Rc::downgrade(&grandparent_parent));
            } else {
                // grandparent is the root, then the parent become new root.
                *root = Some(parent.clone());
                *(parent.borrow_mut().parent_mut()) = None;
            }
            {
                // parent.right = grandparent
                *(parent.borrow_mut().right_mut()) = Some(grandparent.clone());
                // grandparent.parent = parent
                *(grandparent.borrow_mut().parent_mut()) = Some(Rc::downgrade(&parent));
            }
        }
        LocationRel::RR => {
            // parent_left = parent.left, parent.left = None
            let parent_left = parent.borrow_mut().left_mut().take();
            {
                // grandparent.right = parent_left
                *(grandparent.borrow_mut().right_mut()) = parent_left.clone();
            }
            if let Some(ref pl) = parent_left {
                // parent_left.parent = grandparent
                *(pl.borrow_mut().parent_mut()) = Some(Rc::downgrade(&grandparent));
            }
            if let Some(grandparent_parent) = get_parent(&grandparent) {
                if is_left_child(&grandparent) {
                    // grandparent_parent.(left|right) = parent
                    *(grandparent_parent.borrow_mut().left_mut()) = Some(parent.clone());
                } else {
                    *(grandparent_parent.borrow_mut().right_mut()) = Some(parent.clone());
                }
                // parent.parent = grandparent
                *(parent.borrow_mut().parent_mut()) = Some(Rc::downgrade(&grandparent_parent));
            } else {
                // grandparent is the root, then the parent become new root.
                *root = Some(parent.clone());
                *(parent.borrow_mut().parent_mut()) = None;
            }
            {
                // parent.left = grandparent
                *(parent.borrow_mut().left_mut()) = Some(grandparent.clone());
                // grandparent.parent = parent
                *(grandparent.borrow_mut().parent_mut()) = Some(Rc::downgrade(&parent));
            }
        }
        LocationRel::LR => {
            // parent.right = node.left
            let node_left = { node.borrow_mut().left_mut().take() };
            {
                *(parent.borrow_mut().right_mut()) = node_left.clone();
            }
            // node.left.parent = parent
            if let Some(ref nl) = node_left {
                *(nl.borrow_mut().parent_mut()) = Some(Rc::downgrade(&parent));
            }
            // grandparent.left = node.right
            let node_right = { node.borrow_mut().right_mut().take() };
            {
                *(grandparent.borrow_mut().left_mut()) = node_right.clone();
            }
            // node.right.parent = grandparent
            if let Some(ref nr) = node_right {
                *(nr.borrow_mut().parent_mut()) = Some(Rc::downgrade(&grandparent));
            }
            // node.left = parent
            {
                *(node.borrow_mut().left_mut()) = Some(parent.clone());
            }
            // parent.parent = node
            {
                *(parent.borrow_mut().parent_mut()) = Some(Rc::downgrade(&node));
            }
            // node.right = grandparent
            {
                *(node.borrow_mut().right_mut()) = Some(grandparent.clone());
            }

            if let Some(grandparent_parent) = get_parent(&grandparent) {
                if is_left_child(&grandparent) {
                    *(grandparent_parent.borrow_mut().left_mut()) = Some(node.clone());
                } else {
                    *(grandparent_parent.borrow_mut().right_mut()) = Some(node.clone());
                }
                // node.parent = grandparent_parent
                *(node.borrow_mut().parent_mut()) = Some(Rc::downgrade(&grandparent_parent));
            } else {
                *root = Some(node.clone());
                *(node.borrow_mut().parent_mut()) = None;
            }
            // grandparent.parent = node
            {
                *(grandparent.borrow_mut().parent_mut()) = Some(Rc::downgrade(&node));
            }
        }
        LocationRel::RL => {
            // parent.left = node.right
            let node_right = { node.borrow_mut().right_mut().take() };
            {
                *(parent.borrow_mut().left_mut()) = node_right.clone();
            }
            // node.right.parent = parent
            if let Some(ref nr) = node_right {
                *(nr.borrow_mut().parent_mut()) = Some(Rc::downgrade(&parent));
            }
            // grandparent.right = node.left
            let node_left = { node.borrow_mut().left_mut().take() };
            {
                *(grandparent.borrow_mut().right_mut()) = node_left.clone();
            }
            // node.left.parent = grandparent
            if let Some(ref nl) = node_left {
                *(nl.borrow_mut().parent_mut()) = Some(Rc::downgrade(&grandparent));
            }
            // node.left = grandparent
            {
                *(node.borrow_mut().left_mut()) = Some(grandparent.clone());
            }
            // node.right = parent
            {
                *(node.borrow_mut().right_mut()) = Some(parent.clone());
            }
            // parent.parent = node
            {
                *(parent.borrow_mut().parent_mut()) = Some(Rc::downgrade(&node));
            }
            if let Some(grandparent_parent) = get_parent(&grandparent) {
                if is_left_child(&grandparent) {
                    *(grandparent_parent.borrow_mut().left_mut()) = Some(node.clone());
                } else {
                    *(grandparent_parent.borrow_mut().right_mut()) = Some(node.clone());
                }
                // node.parent = grandparent_parent
                *(node.borrow_mut().parent_mut()) = Some(Rc::downgrade(&grandparent_parent));
            } else {
                *root = Some(node.clone());
                *(node.borrow_mut().parent_mut()) = None;
            }
            // grandparent.parent = node
            {
                *(grandparent.borrow_mut().parent_mut()) = Some(Rc::downgrade(&node));
            }
        }
    }
}

// shared Tiny helpers
pub fn get_parent<T, N>(n: &Rc<RefCell<N>>) -> Option<Rc<RefCell<N>>>
where
    T: Ord + Copy,
    N: TreeNode<T>,
{
    n.borrow().parent().as_ref().and_then(|w| w.upgrade())
}

pub fn get_grandparent<T, N>(n: &Rc<RefCell<N>>) -> Option<Rc<RefCell<N>>>
where
    T: Ord + Copy,
    N: TreeNode<T>,
{
    get_parent::<T, N>(n).and_then(|p| get_parent::<T, N>(&p))
}

pub fn is_left_child<T, N>(n: &Rc<RefCell<N>>) -> bool
where
    T: Ord + Copy,
    N: TreeNode<T>,
{
    if let Some(p) = get_parent::<T, N>(n) {
        if let Some(l) = p.borrow().left() {
            Rc::ptr_eq(l, n)
        } else {
            false
        }
    } else {
        false
    }
}
