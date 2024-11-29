//! Grab all elements available via the accessibility tree on Linux.
//! Output the entire tree as JSON.
//!
//! ```sh
//! cargo run > output.json
//! ```
//! Authors:
//!    Luuk van der Duim,
//!    Tait Hoyem

mod indextree_ext;
use indextree_ext::{NodeIdExt, HasRole};
mod role_set;
use role_set::RoleSet;
use atspi_common::Role;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
use std::fs::File;
use std::collections::VecDeque;
use std::time::Instant;
use std::env;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

use indextree::{Arena, NodeId};

#[derive(Debug, Deserialize, Serialize)]
struct Node {
    role: Role,
		roleset: RoleSet,
}
impl HasRole for Node {
	fn roleset(&self) -> RoleSet {
		self.roleset
	}
}
impl Node {
    fn from_a11y_node(node: A11yNode, tree: &mut Arena<Node>) -> NodeId {
        let new_node = Node {
            role: node.role,
						roleset: RoleSet::default(),
        };
        let id = tree.new_node(new_node);
        for child in node.children {
            let child_id = Self::from_a11y_node(child, tree);
            id.append(child_id, tree);
        }
        id
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Tree {
    inner: Arena<Node>,
    root: NodeId,
}
impl Tree {
    fn build_rolesets(&mut self) {
        for leaf_id in self.root.descendants(&self.inner).collect::<Vec<_>>() {
            let leaf_roleset = {
                let leaf = self.inner.get_mut(leaf_id).expect("Valid leaf node").get_mut();
                leaf.roleset |= leaf.role;
								leaf.roleset
            };
            for mut anc_id in leaf_id.ancestors(&self.inner).collect::<Vec<_>>() {
                let anc = self.inner.get_mut(anc_id).expect("Valid ancestor node").get_mut();
                anc.roleset |= leaf_roleset;
            }
        }
    }
    fn from_root_node(root_node: A11yNode) -> Self {
        let mut tree: Arena<Node> = Arena::new();
        let root_id = Node::from_a11y_node(root_node, &mut tree);
        Tree { inner: tree, root: root_id }
    }
    fn leafs(&self) -> impl Iterator<Item = NodeId> + use<'_> {
        self.root
            .descendants(&self.inner)
            .filter(|node| node.children(&self.inner).next().is_none())
    }
    fn nodes(&self) -> usize {
        self.inner.count()
    }
    fn find_first(&self, role: Role) -> Option<NodeId> {
        self.root
            .descendants(&self.inner)
            .find(move |node_id| match self.inner.get(*node_id) {
                None => false,
                Some(node) => node.get().role == role
            })
    }
    fn find_first_no_stack_ext(&self, role: Role) -> Option<NodeId> {
        NodeIdExt::descendants_role(self.root, &self.inner, role.into())
            .find(move |node_id| match self.inner.get(*node_id) {
                None => false,
                Some(node) => node.get().role == role
            })
    }
    fn find_first_roleset(&self, role: Role) -> Option<NodeId> {
        let roles: RoleSet = role.into();
        let mut stack = VecDeque::new();
        stack.reserve(33);
        stack.push_back(self.root);
        while let Some(id) = stack.pop_front() {
            if self.inner.get(id).unwrap().get().role == role {
                return Some(id);
            }
            id.children(&self.inner)
                .rev()
                .filter(|child_id| {
                    let child = self.inner.get(*child_id).unwrap();
                    child.get().roleset.contains(roles)
                })
                .for_each(|good_child| {
                    stack.push_front(good_child);
                });
        }
        None
    }
    fn how_many(&self, role: Role) -> usize {
        self.root
            .descendants(&self.inner)
            .filter_map(move |node_id| self.inner.get(node_id))
            .filter(|node| node.get().role == role)
            .count()
    }
    fn max_depth(&self) -> usize {
        self.root
            .descendants(&self.inner)
            .map(|item| item.ancestors(&self.inner).count())
            .max()
            .expect("A valid ancestors size!")
    }
    fn unique_roles(&self) -> Vec<Role> {
        self.root
            .descendants(&self.inner)
            .filter_map(move |node_id| self.inner.get(node_id))
            .map(|node| node.get().role)
            .fold(Vec::new(), |mut roles, role| {
                if !roles.contains(&role) {
                    roles.push(role);
                }
                roles
            })
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct A11yNode {
	role: Role,
	children: Vec<A11yNode>,
}

#[derive(Clone, Copy)]
pub struct CharSet {
	pub horizontal: char,
	pub vertical: char,
	pub connector: char,
	pub end_connector: char,
}
pub const SINGLE_LINE: CharSet =
	CharSet { horizontal: '─', vertical: '│', connector: '├', end_connector: '└' };

impl Display for A11yNode {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		self.fmt_with(f, SINGLE_LINE, &mut Vec::new())
	}
}

impl A11yNode {
	fn fmt_with(
		&self,
		f: &mut std::fmt::Formatter<'_>,
		style: CharSet,
		prefix: &mut Vec<bool>,
	) -> std::fmt::Result {
		let mut numof = 0;
		let mut max_depth = 0;
		let mut leafs = 0;
		let mut stack: Vec<(&Self, usize, usize)> = vec![(self, 0, 0)];
		while let Some((this, siblings, idx)) = stack.pop() {
			if siblings > 0 {
				prefix.push(idx == siblings - 1);
			}
			numof += 1;
			for (i, is_last_at_i) in prefix.iter().enumerate() {
				// if it is the last portion of the line
				let is_last = i == prefix.len() - 1;
				match (is_last, *is_last_at_i) {
					(true, true) => write!(f, "{}", style.end_connector)?,
					(true, false) => write!(f, "{}", style.connector)?,
					// four spaces to emulate `tree`
					(false, true) => write!(f, "    ")?,
					// three spaces and vertical char
					(false, false) => write!(f, "{}   ", style.vertical)?,
				}
			}

			// two horizontal chars to mimic `tree`
			writeln!(
				f,
				"{}{} {}({})",
				style.horizontal,
				style.horizontal,
				this.role,
				this.children.len()
			)?;

			for (i, child) in this.children.iter().enumerate() {
				stack.push((child, this.children.len(), i));
			}
			if this.children.is_empty() {
				max_depth += 1;
				continue;
			} else {
				leafs += 1;
			}
			prefix.pop();
		}
		Ok(())
	}
}

fn main() -> Result<()> {
	let file_name = env::args().nth(1).expect("Must have at least one argument to binary");
  let data = File::open(file_name)?;
  let a11y_node: A11yNode = serde_json::from_reader(data)?;
  let mut tree = Tree::from_root_node(a11y_node);
  let start = Instant::now();
  tree.build_rolesets();
  let end = Instant::now();
  println!("Took {:?} to build roleset index", end-start);
  println!("Total nodes: {:?}", tree.nodes());
  println!("Tree leafs: {:?}", tree.leafs().count());
  for role in tree.unique_roles() {
      let many = tree.how_many(role);
      let start = Instant::now();
      let first = tree.find_first(role);
      let end = Instant::now();
      let startset = Instant::now();
      let firstset = tree.find_first_roleset(role);
      let endset = Instant::now();
      let startfast = Instant::now();
      let firstfast = tree.find_first_no_stack_ext(role);
      let endfast = Instant::now();
      assert_eq!(first, firstset);
      assert_eq!(firstset, firstfast);
      println!("\t{}: {}", role, many);
			println!("\t\tTime for standard traversal: {:?}", end-start);
			println!("\t\tTime for roleset traversal: {:?}", endset-startset);
			println!("\t\tTime for indextree extention: {:?}", endfast-startfast);
  }
  println!("Max depth: {}", tree.max_depth());

	Ok(())
}
