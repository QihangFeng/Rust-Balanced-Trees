use std::arch::x86_64::_SIDD_POSITIVE_POLARITY;
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::{self, Rc, Weak};

use crate::common::*;

pub type AVLRef<T> = Rc<RefCell<AVLNode<T>>>;
pub type AVLWeak<T> = Weak<RefCell<AVLNode<T>>>;

#[derive(Clone, Copy, Debug)]
enum Dir {
    Left,
    Right,
}

#[derive(Debug)]
pub struct AVLNode<T: Ord + Copy> {
    pub value: T,
    pub height: i32,
    pub parent: Option<AVLWeak<T>>,
    pub left: Option<AVLRef<T>>,
    pub right: Option<AVLRef<T>>,
}

#[derive(Debug, Default)]
pub struct AvlTree<T: Ord + Copy> {
    pub root: Option<AVLRef<T>>,
}

impl<T: Ord + Copy + std::fmt::Debug> AvlTree<T> {
    pub fn new() -> Self {
        Self { root: None }
    }

    // Core functions
    pub fn insert(&mut self, value: T) {
        let mut new_node;

        match &self.root {
            Some(root_node) => {
                let mut current = Rc::clone(root_node);
                let mut last_dir;

                loop {
                    let next_child = {
                        let curr_borrow = current.borrow();
                        if curr_borrow.value == value {
                            println!("The value {:?} already exists.", value);
                            return;
                        } else if curr_borrow.value > value {
                            last_dir = Some(Dir::Left);
                            curr_borrow.left.clone()
                        } else {
                            last_dir = Some(Dir::Right);
                            curr_borrow.right.clone()
                        }
                    };

                    if let Some(child) = next_child {
                        current = child;
                    } else {
                        break;
                    }
                }

                new_node = AVLRef::new(RefCell::new(AVLNode {
                    value,
                    height: 1,
                    parent: Some(Rc::downgrade(&current)),
                    left: None,
                    right: None,
                }));

                match last_dir {
                    Some(Dir::Left) => {
                        current.borrow_mut().left = Some(new_node.clone());
                    }
                    Some(Dir::Right) => {
                        current.borrow_mut().right = Some(new_node.clone());
                    }
                    None => {}
                }

                current = new_node;

                loop {
                    Self::update_height(&current);

                    let balance = Self::balance_factor(&current);

                    // LL
                    if balance > 1 && value < current.borrow().left.as_ref().unwrap().borrow().value
                    {
                        let grandchild = {
                            current
                                .borrow()
                                .left
                                .as_ref()
                                .unwrap()
                                .borrow()
                                .left
                                .as_ref()
                                .unwrap()
                                .clone()
                        };
                        let left_child = current.borrow().left.clone().unwrap();
                        rotate(&mut self.root, &grandchild, LocationRel::LL);
                        Self::update_height(&current);
                        Self::update_height(&left_child);
                        break;
                    }
                    // RR
                    else if balance < -1
                        && value > current.borrow().right.as_ref().unwrap().borrow().value
                    {
                        let grandchild = {
                            current
                                .borrow()
                                .right
                                .as_ref()
                                .unwrap()
                                .borrow()
                                .right
                                .as_ref()
                                .unwrap()
                                .clone()
                        };
                        let right_child = current.borrow().right.clone().unwrap();
                        rotate(&mut self.root, &grandchild, LocationRel::RR);
                        Self::update_height(&current);
                        Self::update_height(&right_child);
                        break;
                    }
                    // LR
                    else if balance > 1
                        && value > current.borrow().left.as_ref().unwrap().borrow().value
                    {
                        let left_child = current.borrow().left.clone().unwrap();
                        let grandchild = left_child.borrow().right.clone().unwrap();
                        rotate(&mut self.root, &grandchild, LocationRel::LR);
                        Self::update_height(&left_child);
                        Self::update_height(&current);
                        Self::update_height(&grandchild);
                        break;
                    }
                    // RL
                    else if balance < -1
                        && value < current.borrow().right.as_ref().unwrap().borrow().value
                    {
                        let right_child = current.borrow().right.clone().unwrap();
                        let grandchild = right_child.borrow().left.clone().unwrap();
                        rotate(&mut self.root, &grandchild, LocationRel::RL);
                        Self::update_height(&right_child);
                        Self::update_height(&current);
                        Self::update_height(&grandchild);
                        break;
                    }

                    let parent_opt = current.borrow().parent.clone();

                    if let Some(weak_p) = parent_opt {
                        if let Some(parent) = weak_p.upgrade() {
                            current = parent;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                println!("The value {:?} is inserted successfully!", value);
            }

            None => {
                new_node = AVLRef::new(RefCell::new(AVLNode {
                    value: value,
                    height: 1,
                    parent: None,
                    left: None,
                    right: None,
                }));

                self.root = Some(new_node);
                println!("The value {:?} is inserted successfully!", value);
            }
        }
    }

    pub fn delete(&mut self, value: T) {
        let mut search = self.root.clone();
        let mut target: Option<AVLRef<T>> = None;
        while let Some(node) = search {
            if node.borrow().value == value {
                target = Some(node);
                break;
            } else if value < node.borrow().value {
                search = node.borrow().left.clone();
            } else {
                search = node.borrow().right.clone();
            }
        }

        let mut node = match target {
            Some(n) => n,
            None => {
                println!("The value {:?} is not found.", value);
                return;
            } // value not found
        };

        if node.borrow().left.is_some() && node.borrow().right.is_some() {
            let mut pred = node.borrow().left.clone().unwrap();
            loop {
                let right_opt = {
                    let pred_borrow = pred.borrow();
                    pred_borrow.right.clone()
                };
                if let Some(right) = right_opt {
                    pred = right;
                } else {
                    break;
                }
            }
            let pred_val = pred.borrow().value;
            node.borrow_mut().value = pred_val;
            node = pred;
        }

        let child = if node.borrow().left.is_some() {
            node.borrow().left.clone()
        } else {
            node.borrow().right.clone()
        };

        if let Some(c) = &child {
            c.borrow_mut().parent = node.borrow().parent.clone();
        }

        let parent_weak = node.borrow().parent.clone();

        if let Some(pw) = parent_weak.as_ref() {
            if let Some(parent) = pw.upgrade() {
                let mut parent_mut = parent.borrow_mut();
                if parent_mut
                    .left
                    .as_ref()
                    .map(|n| Rc::ptr_eq(n, &node))
                    .unwrap_or(false)
                {
                    parent_mut.left = child.clone();
                } else if parent_mut
                    .right
                    .as_ref()
                    .map(|n| Rc::ptr_eq(n, &node))
                    .unwrap_or(false)
                {
                    parent_mut.right = child.clone();
                }
            }
        } else {
            self.root = child.clone();
            if let Some(c) = &self.root {
                c.borrow_mut().parent = None;
            }
        }

        let mut current_opt = parent_weak.and_then(|pw| pw.upgrade());
        while let Some(current) = current_opt {
            Self::update_height(&current);

            let balance = Self::balance_factor(&current);

            // LL
            if balance > 1 && Self::balance_factor(&current.borrow().left.as_ref().unwrap()) >= 0 {
                let grandchild = {
                    current
                        .borrow()
                        .left
                        .as_ref()
                        .unwrap()
                        .borrow()
                        .left
                        .as_ref()
                        .unwrap()
                        .clone()
                };
                let left_child = current.borrow().left.clone().unwrap();
                rotate(&mut self.root, &grandchild, LocationRel::LL);
                Self::update_height(&current);
                Self::update_height(&left_child);
            }
            // RR
            else if balance < -1
                && Self::balance_factor(&current.borrow().right.as_ref().unwrap()) <= 0
            {
                let grandchild = {
                    current
                        .borrow()
                        .right
                        .as_ref()
                        .unwrap()
                        .borrow()
                        .right
                        .as_ref()
                        .unwrap()
                        .clone()
                };
                let right_child = current.borrow().right.clone().unwrap();
                rotate(&mut self.root, &grandchild, LocationRel::RR);
                Self::update_height(&current);
                Self::update_height(&right_child);
            }
            // LR
            else if balance > 1
                && Self::balance_factor(&current.borrow().left.as_ref().unwrap()) < 0
            {
                let left_child = current.borrow().left.clone().unwrap();
                let grandchild = left_child.borrow().right.clone().unwrap();
                rotate(&mut self.root, &grandchild, LocationRel::LR);
                Self::update_height(&left_child);
                Self::update_height(&current);
                Self::update_height(&grandchild);
            }
            // RL
            else if balance < -1
                && Self::balance_factor(&current.borrow().right.as_ref().unwrap()) > 0
            {
                let right_child = current.borrow().right.clone().unwrap();
                let grandchild = right_child.borrow().left.clone().unwrap();
                rotate(&mut self.root, &grandchild, LocationRel::RL);
                Self::update_height(&right_child);
                Self::update_height(&current);
                Self::update_height(&grandchild);
            }

            current_opt = current.borrow().parent.clone().and_then(|pw| pw.upgrade());
        }
        println!("The value {:?} is deleted successfully!", value);
    }

    // Shared functions used by insert/delete
    fn height_of_node(node: &Option<AVLRef<T>>) -> i32 {
        node.as_ref().map_or(0, |n| n.borrow().height)
    }

    fn update_height(node: &AVLRef<T>) {
        let left_h = Self::height_of_node(&node.borrow().left);
        let right_h = Self::height_of_node(&node.borrow().right);
        node.borrow_mut().height = (left_h.max(right_h)) + 1;
    }

    fn recompute_heights(node: &Option<AVLRef<T>>) -> i32 {
        if let Some(rc) = node {
            let left_h = Self::recompute_heights(&rc.borrow().left);
            let right_h = Self::recompute_heights(&rc.borrow().right);
            let h = left_h.max(right_h) + 1;
            rc.borrow_mut().height = h;
            h
        } else {
            0
        }
    }

    fn balance_factor(node: &AVLRef<T>) -> i32 {
        Self::height_of_node(&node.borrow().left) - Self::height_of_node(&node.borrow().right)
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
impl<T: Ord + Copy + Debug> TreeNode<T> for AVLNode<T> {
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
    fn height(&self) -> Option<usize> {
        Some(self.height as usize)
    }

    fn to_string(&self) -> String {
        return format!("{:?} [height: {}]", self.value, self.height);
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
impl<T: Ord + Copy + std::fmt::Debug> TreeOps<T> for AvlTree<T> {
    fn insert(&mut self, v: Vec<T>) {
        for x in v {
            AvlTree::insert(self, x);
        }
    }

    fn delete(&mut self, v: Vec<T>) {
        for x in v {
            AvlTree::delete(self, x);
        }
    }

    fn count_leaves(&self) -> usize {
        count_leaves(&self.root)
    }

    fn height(&self) -> usize {
        height(&self.root)
    }

    fn inorder_traversal(&self) {
        println!("AVL Tree (in-order):");
        inorder_traversal(&self.root);
    }

    fn is_tree_empty(&self) -> bool {
        self.root.is_none()
    }

    fn print_tree_pattern1(&self) {
        println!("AVL Tree (Vertical):");
        print_tree_pattern1(&self.root);
    }

    fn print_tree_pattern2(&self) {
        println!("AVL Tree (Horizontal):");
        print_tree_pattern2(&self.root);
    }

    fn clear(&mut self) {
        self.root = None;
    }
}
