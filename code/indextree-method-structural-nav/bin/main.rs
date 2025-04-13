use indextree_method_structural_nav::{A11yNode, Tree, TreeCount, TreeTraversal};

use std::env;
use std::fs;
use std::time::Instant;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let file_name = env::args()
        .nth(1)
        .expect("Must have at least one argument to binary");
    let read_start = Instant::now();
    let data = fs::read_to_string(file_name).expect("Should be able to read file!");
    let read_end = Instant::now();
    println!("Took {:?} to read file", read_end - read_start);
    let json_start = Instant::now();
    let a11y_node: A11yNode = serde_json::from_str(&data)?;
    let json_end = Instant::now();
    println!("Took {:?} to parse JSON", json_end - json_start);
    let mut tree = Tree::from_root_node(a11y_node.clone());
    let mut tree_count = TreeCount::from_root_node(a11y_node.clone());
    let start = Instant::now();
    tree.build_rolesets();
    let end = Instant::now();
    println!("Took {:?} to build bitset roleset index", end - start);
    let startcount = Instant::now();
    tree_count.build_rolesets();
    let endcount = Instant::now();
    println!(
        "Took {:?} to build count roleset index",
        endcount - startcount
    );
    println!("Total nodes: {:?}", tree.nodes());
    println!("Tree leafs: {:?}", tree.iter_leafs().count());
    for role in tree.unique_roles().role_iter() {
        {
            let many = tree.how_many(role);
            let start = Instant::now();
            let first = tree.find_first(role);
            let end = Instant::now();
            let startset = Instant::now();
            let firstset = tree.find_first_stack(role);
            let endset = Instant::now();
            let startfast = Instant::now();
            let firstfast = tree.find_first_roleset(role);
            let endfast = Instant::now();
            assert_eq!(first, firstset);
            assert_eq!(firstset, firstfast);
            println!("\t{}: {}", role, many);
            println!("\t\tTime for standard traversal: {:?}", end - start);
            println!("\t\tTime for roleset traversal: {:?}", endset - startset);
            println!(
                "\t\tTime for indextree extention: {:?}",
                endfast - startfast
            );
        }
        {
            let many = tree_count.how_many(role);
            let start = Instant::now();
            let first = tree_count.find_first(role);
            let end = Instant::now();
            let startset = Instant::now();
            let firstset = tree_count.find_first_stack(role);
            let endset = Instant::now();
            let startfast = Instant::now();
            let firstfast = tree_count.find_first_roleset(role);
            let endfast = Instant::now();
            assert_eq!(first, firstset);
            assert_eq!(firstset, firstfast);
            println!("\t{}: {}", role, many);
            println!("\t\tTime for standard traversal: {:?}", end - start);
            println!("\t\tTime for roleset traversal: {:?}", endset - startset);
            println!(
                "\t\tTime for indextree extention: {:?}",
                endfast - startfast
            );
        }
    }
    println!("Max depth: {}", tree.max_depth());

    Ok(())
}
