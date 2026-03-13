use std::cell::RefCell;
use std::clone;
use std::cmp::Ordering::*;
use std::fmt::Debug;
use std::rc::{Rc, Weak};
use std::thread::park_timeout_ms;

use crate::common::*;

pub type RBRef<T> = Rc<RefCell<RBNode<T>>>;
pub type RBWeak<T> = Weak<RefCell<RBNode<T>>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Red,
    Black,
}

#[derive(Debug)]
pub struct RBNode<T: Ord + Copy> {
    pub value: T,
    pub color: Color,
    pub parent: Option<RBWeak<T>>,
    pub left: Option<RBRef<T>>,
    pub right: Option<RBRef<T>>,
}

#[derive(Debug, Default)]
pub struct RedBlackTree<T: Ord + Copy> {
    pub root: Option<RBRef<T>>,
}

impl<T: Ord + Copy + std::fmt::Debug> RedBlackTree<T> {
    pub fn new() -> Self {
        Self { root: None }
    }

    // core functions
    pub fn insert(&mut self, value: T) {
        let node = Rc::new(RefCell::new(RBNode {
            value,
            // new node's initial color is red
            color: Color::Red,
            parent: None,
            left: None,
            right: None,
        }));
        // Find the location and add the new node
        match find_location(&self.root, value) {
            // If empty, the new node is the root and should be black.
            InsertionPoint::AsRoot => {
                self.root = Some(node.clone());
                node.borrow_mut().color = Color::Black;
                println!("The value {:?} is inserted successfully!", value);
                return;
            }
            // If the value is duplicate, then ignore it.
            InsertionPoint::Existing(_n) => {
                println!("The value {:?} already exists.", value);
                return;
            }
            InsertionPoint::AtLeftOf(p) => {
                // Insert the new node as the left child of p
                p.borrow_mut().left = Some(node.clone());
                // Set the new node's parent
                // Rc::downgrade(&p) converts the Strong Rc into Weak
                node.borrow_mut().parent = Some(Rc::downgrade(&p));
                // If the parent is black, then we don't need to rebalance.
            }
            InsertionPoint::AtRightOf(p) => {
                p.borrow_mut().right = Some(node.clone());
                node.borrow_mut().parent = Some(Rc::downgrade(&p));
            }
        }
        self.rebalance_insert(node);
        println!("The value {:?} is inserted successfully!", value);
    }

    pub fn delete(&mut self, value: T) {
        use std::cmp::Ordering;

        // 1. Search for the node z that contains the target value.
        let mut cur = self.root.clone();
        let mut z: Option<RBRef<T>> = None;

        while let Some(node) = cur {
            let v = node.borrow().value;
            match value.cmp(&v) {
                Ordering::Equal => {
                    z = Some(node.clone());
                    break;
                }
                Ordering::Less => cur = node.borrow().left.clone(),
                Ordering::Greater => cur = node.borrow().right.clone(),
            }
        }

        let z = match z {
            Some(n) => n,
            None => {
                println!("The value {:?} does not exist.", value);
                return;
            }
        };

        // y is the actual node being removed (z or z’s successor).
        let mut y = z.clone();
        let mut y_original_color = y.borrow().color;

        // x replaces y; x_parent is needed when x is None (NIL child).
        let mut x: Option<RBRef<T>> = None;
        let mut x_parent: Option<RBRef<T>> = None;

        // 2. Standard BST removal: reduce to a case with at most one non-null child.
        if z.borrow().left.is_none() {
            // Only right child (or none)
            let right_child = { z.borrow_mut().right.take() };
            x_parent = get_parent::<T, RBNode<T>>(&z);
            x = right_child.clone();
            self.transplant(&z, right_child);
        } else if z.borrow().right.is_none() {
            // Only left child
            let left_child = { z.borrow_mut().left.take() };
            x_parent = get_parent::<T, RBNode<T>>(&z);
            x = left_child.clone();
            self.transplant(&z, left_child);
        } else {
            // Two children: find predecessor (maximum of left subtree)
            let z_left = z.borrow().left.clone();
            let z_right = z.borrow().right.clone();

            let mut pred = {
                let mut cur = z.borrow().left.as_ref().unwrap().clone();
                loop {
                    let right = cur.borrow().right.clone();
                    if let Some(r) = right {
                        cur = r;
                    } else {
                        break cur;
                    }
                }
            };
            // y(pred) is the real deleted node
            y = pred.clone();
            y_original_color = pred.borrow().color;

            // pred only have two case: no child or only a left child
            let pred_left = { pred.borrow_mut().left.take() };
            x = pred_left.clone();

            let pred_parent = get_parent::<T, RBNode<T>>(&pred).unwrap();

            if Rc::ptr_eq(&pred_parent, &z) {
                // Case A: pred is the direct right child of z
                x_parent = Some(pred.clone());

                self.transplant(&z, Some(pred.clone()));

                {
                    let mut pm = pred.borrow_mut();

                    pm.left = pred_left.clone();
                    if let Some(ref l) = pm.left {
                        l.borrow_mut().parent = Some(Rc::downgrade(&pred));
                    }

                    pm.right = z_right;
                    if let Some(ref r) = pm.right {
                        r.borrow_mut().parent = Some(Rc::downgrade(&pred));
                    }

                    pm.color = z.borrow().color;
                }
            } else {
                // Case B: pred is located at the deeper position of right subtree
                self.transplant(&pred, pred_left.clone());
                x_parent = Some(pred_parent.clone());

                self.transplant(&z, Some(pred.clone()));

                {
                    let mut pm = pred.borrow_mut();

                    pm.left = z_left;
                    if let Some(ref l) = pm.left {
                        l.borrow_mut().parent = Some(Rc::downgrade(&pred));
                    }

                    pm.right = z_right;
                    if let Some(ref r) = pm.right {
                        r.borrow_mut().parent = Some(Rc::downgrade(&pred));
                    }

                    pm.color = z.borrow().color;
                }
            }
        }

        // 3. If the removed node was black, RB properties may be violated → fix delete.
        if y_original_color == Color::Black {
            self.rebalance_delete(x, x_parent);
        }

        println!("The value {:?} is deleted successfully!", value);
    }

    fn transplant(&mut self, u: &RBRef<T>, v: Option<RBRef<T>>) {
        // Replaces subtree rooted at u with subtree rooted at v.

        let parent = get_parent::<T, RBNode<T>>(u);

        if let Some(p) = parent {
            if is_left_child::<T, RBNode<T>>(u) {
                p.borrow_mut().left = v.clone();
            } else {
                p.borrow_mut().right = v.clone();
            }
            if let Some(ref vv) = v {
                vv.borrow_mut().parent = Some(Rc::downgrade(&p));
            }
        } else {
            // u is root
            self.root = v.clone();
            if let Some(ref vv) = v {
                vv.borrow_mut().parent = None;
            }
        }
    }

    // Shared functions used by insert/delete
    fn rebalance_insert(&mut self, mut node: RBRef<T>) {
        // If the parent is black, then return.
        if let Some(parent) = get_parent(&node) {
            if parent.borrow().color == Color::Black {
                return;
            }
        } else {
            // If the node doesn't have parent -> root
            node.borrow_mut().color = Color::Black;
            return;
        }
        // If parent is red, modify the tree according to the color of uncle.
        let grandparent = get_grandparent(&node).unwrap();
        let parent = get_parent(&node).unwrap();
        let parent_is_left = is_left_child(&parent);
        let uncle = if parent_is_left {
            grandparent.borrow().right.clone()
        } else {
            grandparent.borrow().left.clone()
        };

        let uncle_is_red = uncle
            .as_ref()
            .map(|n| n.borrow().color == Color::Red)
            .unwrap_or(false);
        // Red uncle -> recolor and consider the original grandparent as the new node, rebalance again.
        if uncle_is_red {
            Self::recolor(&grandparent);
            Self::recolor(&parent);
            Self::recolor(&uncle.unwrap());
            self.rebalance_insert(grandparent);
            return;
        } else {
            // Black uncle -> rotate and recolor
            // Note: if uncle is None, then it's also consider as black.
            let location_rel =
                find_location_rel(&node).expect("Parent and grandparent must exist!");
            match location_rel {
                LocationRel::LL => {
                    rotate(&mut self.root, &node, LocationRel::LL);
                    Self::recolor(&grandparent);
                    Self::recolor(&parent);
                }
                LocationRel::RR => {
                    rotate(&mut self.root, &node, LocationRel::RR);
                    Self::recolor(&grandparent);
                    Self::recolor(&parent);
                }
                LocationRel::RL => {
                    rotate(&mut self.root, &node, LocationRel::RL);
                    Self::recolor(&grandparent);
                    Self::recolor(&node);
                }
                LocationRel::LR => {
                    rotate(&mut self.root, &node, LocationRel::LR);
                    Self::recolor(&grandparent);
                    Self::recolor(&node);
                }
            }
        }
    }

    fn rebalance_delete(&mut self, mut x: Option<RBRef<T>>, mut x_parent: Option<RBRef<T>>) {
        // x == None effectively represents a black NIL child.
        while (x.is_none() || Self::node_color(&x) == Color::Black)
            && x_parent.is_some()
            && self.root.is_some()
        {
            let parent = x_parent.as_ref().unwrap().clone();

            // Determine whether the “double black” node x is the left or right child.
            let x_is_left = match x.as_ref() {
                Some(n) => is_left_child::<T, RBNode<T>>(n),
                None => parent.borrow().left.is_none(), // if x == None, check which side is empty
            };

            if x_is_left {
                // Sibling w is on the right side
                let mut w_opt = parent.borrow().right.clone();
                if w_opt.is_none() {
                    // No sibling: propagate double-black upward
                    x = Some(parent.clone());
                    x_parent = get_parent::<T, RBNode<T>>(&parent);
                    continue;
                }
                let w = w_opt.as_ref().unwrap().clone();

                // Case 1: sibling is red
                if Self::node_color(&w_opt) == Color::Red {
                    w.borrow_mut().color = Color::Black;
                    parent.borrow_mut().color = Color::Red;
                    self.left_rotate(&parent);
                    w_opt = parent.borrow().right.clone();
                }

                let w = w_opt.as_ref().unwrap().clone();
                let w_left_color = Self::node_color(&w.borrow().left.clone());
                let w_right_color = Self::node_color(&w.borrow().right.clone());

                // Case 2: sibling is black and both nephews are black
                if w_left_color == Color::Black && w_right_color == Color::Black {
                    w.borrow_mut().color = Color::Red;
                    x = Some(parent.clone());
                    x_parent = get_parent::<T, RBNode<T>>(&parent);
                } else {
                    // Case 3 or 4:
                    if w_right_color == Color::Black {
                        // Case 3: far nephew is black, near nephew is red
                        if let Some(ref wl) = w.borrow().left.clone() {
                            wl.borrow_mut().color = Color::Black;
                        }
                        w.borrow_mut().color = Color::Red;
                        self.right_rotate(&w);
                        w_opt = parent.borrow().right.clone();
                    }

                    // Case 4: far nephew is red
                    let w = w_opt.as_ref().unwrap().clone();
                    w.borrow_mut().color = parent.borrow().color;
                    parent.borrow_mut().color = Color::Black;
                    if let Some(ref wr) = w.borrow().right.clone() {
                        wr.borrow_mut().color = Color::Black;
                    }
                    self.left_rotate(&parent);
                    x = self.root.clone();
                    x_parent = None;
                }
            } else {
                // Symmetric case: x is the right child, sibling is on the left
                let mut w_opt = parent.borrow().left.clone();
                if w_opt.is_none() {
                    x = Some(parent.clone());
                    x_parent = get_parent::<T, RBNode<T>>(&parent);
                    continue;
                }
                let w = w_opt.as_ref().unwrap().clone();

                if Self::node_color(&w_opt) == Color::Red {
                    w.borrow_mut().color = Color::Black;
                    parent.borrow_mut().color = Color::Red;
                    self.right_rotate(&parent);
                    w_opt = parent.borrow().left.clone();
                }

                let w = w_opt.as_ref().unwrap().clone();
                let w_left_color = Self::node_color(&w.borrow().left.clone());
                let w_right_color = Self::node_color(&w.borrow().right.clone());

                if w_left_color == Color::Black && w_right_color == Color::Black {
                    w.borrow_mut().color = Color::Red;
                    x = Some(parent.clone());
                    x_parent = get_parent::<T, RBNode<T>>(&parent);
                } else {
                    if w_left_color == Color::Black {
                        if let Some(ref wr) = w.borrow().right.clone() {
                            wr.borrow_mut().color = Color::Black;
                        }
                        w.borrow_mut().color = Color::Red;
                        self.left_rotate(&w);
                        w_opt = parent.borrow().left.clone();
                    }

                    let w = w_opt.as_ref().unwrap().clone();
                    w.borrow_mut().color = parent.borrow().color;
                    parent.borrow_mut().color = Color::Black;
                    if let Some(ref wl) = w.borrow().left.clone() {
                        wl.borrow_mut().color = Color::Black;
                    }
                    self.right_rotate(&parent);
                    x = self.root.clone();
                    x_parent = None;
                }
            }
        }

        if let Some(ref n) = x {
            n.borrow_mut().color = Color::Black;
        }
    }

    //Tiny helpers

    fn left_rotate(&mut self, x: &RBRef<T>) {
        // Standard RB-tree left rotation around node x.
        // y is the right child of x.
        let y = {
            let mut xm = x.borrow_mut();
            xm.right.take().expect("left_rotate: x.right is None")
        };

        // Move y.left to x.right
        {
            let mut ym = y.borrow_mut();
            let y_left = ym.left.take();
            {
                let mut xm = x.borrow_mut();
                xm.right = y_left.clone();
                if let Some(ref r) = xm.right {
                    r.borrow_mut().parent = Some(Rc::downgrade(x));
                }
            }
        }

        // Attach y to x's parent
        let x_parent = get_parent::<T, RBNode<T>>(x);
        if let Some(ref p) = x_parent {
            if is_left_child::<T, RBNode<T>>(x) {
                p.borrow_mut().left = Some(y.clone());
            } else {
                p.borrow_mut().right = Some(y.clone());
            }
            y.borrow_mut().parent = Some(Rc::downgrade(p));
        } else {
            // x was root → y becomes new root
            self.root = Some(y.clone());
            y.borrow_mut().parent = None;
        }

        // Put x as y.left
        {
            let mut ym = y.borrow_mut();
            ym.left = Some(x.clone());
        }
        x.borrow_mut().parent = Some(Rc::downgrade(&y));
    }

    fn right_rotate(&mut self, x: &RBRef<T>) {
        // Standard RB-tree right rotation around node x.
        // y is the left child of x.
        let y = {
            let mut xm = x.borrow_mut();
            xm.left.take().expect("right_rotate: x.left is None")
        };

        // Move y.right to x.left
        {
            let mut ym = y.borrow_mut();
            let y_right = ym.right.take();
            {
                let mut xm = x.borrow_mut();
                xm.left = y_right.clone();
                if let Some(ref l) = xm.left {
                    l.borrow_mut().parent = Some(Rc::downgrade(x));
                }
            }
        }

        // Attach y to x's parent
        let x_parent = get_parent::<T, RBNode<T>>(x);
        if let Some(ref p) = x_parent {
            if is_left_child::<T, RBNode<T>>(x) {
                p.borrow_mut().left = Some(y.clone());
            } else {
                p.borrow_mut().right = Some(y.clone());
            }
            y.borrow_mut().parent = Some(Rc::downgrade(p));
        } else {
            // x was root → y becomes new root
            self.root = Some(y.clone());
            y.borrow_mut().parent = None;
        }

        // Put x as y.right
        {
            let mut ym = y.borrow_mut();
            ym.right = Some(x.clone());
        }
        x.borrow_mut().parent = Some(Rc::downgrade(&y));
    }

    fn node_color(node: &Option<RBRef<T>>) -> Color {
        // Treat None (NIL) as black.
        match node {
            Some(rc) => rc.borrow().color,
            None => Color::Black,
        }
    }

    fn recolor(n: &RBRef<T>) {
        let current_color = n.borrow().color;
        let new_color = match current_color {
            Color::Black => Color::Red,
            Color::Red => Color::Black,
        };
        {
            n.borrow_mut().color = new_color;
        }
    }

    pub fn contains(&self, key: T) -> bool {
        let mut cur = self.root.clone();
        while let Some(n) = cur {
            let nb = n.borrow();
            if key < nb.value {
                cur = nb.left.clone();
            } else if key > nb.value {
                cur = nb.right.clone();
            } else {
                return true;
            }
        }
        false
    }
}

// common::TreeNode implementation
impl<T: Ord + Copy + Debug> TreeNode<T> for RBNode<T> {
    fn value(&self) -> &T {
        &self.value
    }
    fn left(&self) -> &Option<Rc<RefCell<Self>>> {
        &self.left
    }
    fn right(&self) -> &Option<Rc<RefCell<Self>>> {
        &self.right
    }
    fn parent(&self) -> &Option<Weak<RefCell<Self>>> {
        &self.parent
    }
    fn color(&self) -> Option<DisplayColor> {
        Some(match self.color {
            Color::Red => DisplayColor::Red,
            Color::Black => DisplayColor::Black,
        })
    }

    fn to_string(&self) -> String {
        let s = match self.color {
            Color::Red => format!("{}{:?} [red]{}", "\x1b[31m", self.value, "\x1b[0m"),
            Color::Black => format!("{}{:?} [black]{}", "\x1b[90m", self.value, "\x1b[0m"),
        };
        return s;
    }

    fn left_mut(&mut self) -> &mut Option<Rc<RefCell<Self>>> {
        &mut self.left
    }
    fn right_mut(&mut self) -> &mut Option<Rc<RefCell<Self>>> {
        &mut self.right
    }
    fn parent_mut(&mut self) -> &mut Option<Weak<RefCell<Self>>> {
        &mut self.parent
    }
}

// ops 3-7
impl<T: Ord + Copy + std::fmt::Debug> TreeOps<T> for RedBlackTree<T> {
    fn insert(&mut self, v: Vec<T>) {
        for x in v {
            RedBlackTree::insert(self, x);
        }
    }

    fn delete(&mut self, v: Vec<T>) {
        for x in v {
            RedBlackTree::delete(self, x);
        }
    }

    fn count_leaves(&self) -> usize {
        count_leaves(&self.root)
    }

    fn height(&self) -> usize {
        height(&self.root)
    }

    fn inorder_traversal(&self) {
        println!("Red-Black Tree (in-order):");
        inorder_traversal(&self.root);
        print!("\n");
    }

    fn is_tree_empty(&self) -> bool {
        self.root.is_none()
    }

    fn print_tree_pattern1(&self) {
        println!("Red-Black Tree (Vertical):");
        print_tree_pattern1(&self.root);
    }

    fn print_tree_pattern2(&self) {
        println!("Red-Black Tree (Horizontal):");
        print_tree_pattern2(&self.root);
    }

    fn clear(&mut self) {
        self.root = None;
    }
}
