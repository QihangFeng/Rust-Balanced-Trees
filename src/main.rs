use std::{env::temp_dir, fmt::Debug, io};

use project2::{avl::AvlTree, common::TreeOps, red_black::RedBlackTree};

fn main() {
    let mut rb_tree: RedBlackTree<i32> = RedBlackTree::new();
    let mut avl_tree: AvlTree<i32> = AvlTree::new();
    loop {
        println!("\n====== Balanced Binary Tree Main Menu ======");
        println!("1. Red-Black Tree");
        println!("2. AVL Tree");
        println!("3. Exit");
        println!("============================================");

        let tree_type = get_input("Select the type of tree: ");
        match tree_type.as_str() {
            "1" => tree_menu("Red-Black", &mut rb_tree),
            "2" => tree_menu("AVL", &mut avl_tree),
            "3" => {
                println!("Thanks for using. Goodbye!👋");
                break;
            }
            _ => println!("Invalid input, please try again."),
        }
    }
}

fn tree_menu<T>(tree_type: &str, tree: &mut T)
where
    T: TreeOps<i32> + Debug,
{
    loop {
        if tree_type == "AVL" {
            println!("\n================= {tree_type} Menu =================");
        } else {
            println!("\n============== {tree_type} Menu ==============");
        }
        println!(" 1. Insert a node");
        println!(" 2. Delete a node");
        println!(" 3. Count leaves");
        println!(" 4. Get tree height");
        println!(" 5. Print In-order traversal");
        println!(" 6. Check empty");
        println!(" 7. Print tree structure");
        println!(" 8. Clear tree");
        println!(" 9. Back");
        println!("10. Exit");
        println!("============================================");

        let op = get_input("Enter the number of operation: ");
        match op.as_str() {
            "1" => {
                let val = get_input("Enter node value to insert: ");
                println!("--> Inserting node with value {val} into {tree_type} Tree...");
                let parsed: Result<Vec<i32>, _> =
                    val.split_whitespace().map(|v| v.parse::<i32>()).collect();
                // TODO: insert() function
                match parsed {
                    Ok(values) if !values.is_empty() => {
                        tree.insert(values);
                    }
                    Ok(_) => {
                        eprintln!("No values entered!");
                    }
                    Err(_) => {
                        eprintln!(
                            // This sequence can test all situations: 20 10 5 30 3 25 4 8 23
                            "Invalid integer in input! Please enter numbers like: 20 10 5 30 3 25 4 8 23"
                        );
                    }
                }
            }
            "2" => {
                let val = get_input("Enter node value to delete: ");
                println!("--> Deleting node with value {val} from {tree_type} Tree...");
                let parsed: Result<Vec<i32>, _> =
                    val.split_whitespace().map(|v| v.parse::<i32>()).collect();
                // TODO: insert() function
                match parsed {
                    Ok(values) if !values.is_empty() => {
                        tree.delete(values);
                    }
                    Ok(_) => {
                        eprintln!("No values entered!");
                    }
                    Err(_) => {
                        eprintln!(
                            // This sequence can test all situations: 20 10 5 30 3 25 4 8 23
                            "Invalid integer in input! Please enter numbers like: 20 10 5 30 3 25 4 8 23"
                        );
                    }
                }
            }
            "3" => {
                println!("--> Counting the number of leaves in {tree_type} Tree...");
                // TODO: count_leaves()
                let h = tree.count_leaves();
                println!("The {tree_type} Tree has {h} leaves.")
            }
            "4" => {
                println!("--> Getting the height of the {tree_type} Tree...");
                // TODO: height()
                let h = tree.height();
                println!("The height of {tree_type} is {h}.");
            }
            "5" => {
                println!("--> Printing in-order traversal of {tree_type} Tree...");
                // TODO: inorder_traversal()
                if tree.is_tree_empty() {
                    println!("The {tree_type} Tree is empty.");
                } else {
                    tree.inorder_traversal();   
                }
            }
            "6" => {
                println!("--> Checking if {tree_type} Tree is empty...");
                // TODO: is_tree_empty()
                if tree.is_tree_empty() {
                    println!("The {tree_type} Tree is empty.");
                } else {
                    println!("The {tree_type} Tree is not empty.")
                }
            }
            "7" => {
                println!("--> Printing full structure of {tree_type} Tree...");
                // TODO: print_tree()
                // println!("{:#?}", tree);
                loop {
                    println!("--------------------------------------------");
                    println!("Choose print pattern:");
                    println!("1. Vertical pattern");
                    println!("2. Horizontal pattern");
                    println!("3. Back");
                    println!("--------------------------------------------");
                    let pat = get_input("Enter pattern (1 or 2): ");
                    match pat.as_str() {
                        "1" => {
                            tree.print_tree_pattern1();
                        }
                        "2" => {
                            tree.print_tree_pattern2();
                        }
                        "3" => break,
                        _ => println!("Invalid input, please try again."),
                    }
                }
            }
            "8" => {
                // clear the tree
                println!("--> Clearing {tree_type} Tree...");
                tree.clear();
                println!("{tree_type} Tree cleared. It is now empty.");
            }
            "9" => {
                println!("Returning to main menu...");
                break;
            }
            "10" => {
                println!("Thanks for using. Goodbye!👋");
                std::process::exit(0);
            }
            _ => println!("Invalid input, please try again."),
        }
    }
}

fn get_input(prompt: &str) -> String {
    use std::io::Write;
    let mut input = String::new();
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}
