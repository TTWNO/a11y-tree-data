//! # `indextree_method_structural_nav`
//!
//! This create is an experiment on how to increase the speed of [screen
//! reader](https://en.wikipedia.org/wiki/Screen_reader) structural navigation commands useing
//! role-based bitset propogation.
//!
//! Internally, this uses an arena-allocated tree and various methods of access to find the ideal
//! method for use in the [Odilia screen reader project](https://odilia.app/).
//!
//! Most functions will have two variants:
//!
//! - `method_name`: sequential arena-allocated tree accessor
//! - `par_method_name`: parallel arena-allocated tree accessor
//!
//! Some have an additional two:
//!
//! - `method_name_roleset`: sequential arena-allocated tree accessor, that uses the bitset
//!   propogation fields to ignore un-needed subtrees.
//! - `par_method_name_roleset`: parallel arena-allocated tree accessor, that uses the bitset
//!   propogation fields to ignore un-needed subtrees.
//!
//! At this time, we only benchmark _accessors_ and not _modifications_.
//! This is planned for the future, as accessibility trees are written _much_ more often than they
//! are read.
//!
//! Check the benchmarks for results.
//!
#![deny(clippy::all, clippy::pedantic, unsafe_code, missing_docs, rustdoc::all)]

mod indextree_ext;
pub use indextree_ext::{HasRole, NodeIdExt};
mod role_set;
use atspi_common::Role;
use rayon::iter::walk_tree;
use rayon::prelude::*;
pub use role_set::{RoleSet, RoleSetVecCount};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fmt::{self, Display, Formatter};
use tap::Tap;

use indextree::{Arena, NodeId};

/// A node containing a role, a roleset for all descendants, and a count of how many of each role
/// in all descendants.
#[derive(Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct NodeCount {
    /// The node's role.
    role: Role,
    /// All descendants' roles and how many.
    roleset: RoleSetVecCount,
}
impl HasRole for NodeCount {
    fn roleset(&self) -> RoleSet {
        self.roleset.1
    }
}

impl NodeCount {
    /// Adds the created [`NodeCount`] to a given arena; returns its new [`NodeId`].
    fn from_a11y_node(node: A11yNode, tree: &mut Arena<NodeCount>) -> NodeId {
        let new_node = NodeCount {
            role: node.role,
            roleset: RoleSetVecCount::default(),
        };
        let id = tree.new_node(new_node);
        for child in node.children {
            let child_id = Self::from_a11y_node(child, tree);
            id.append(child_id, tree);
        }
        id
    }
}

/// Tree traversal mechanisms using a backing [`Arena`] allocator.
pub trait TreeTraversal {
    /// The underlying `Node` type.
    ///
    /// Not to be confused with [`indextree::Node`] which is a wrapper around the inner node type.
    type Node: HasRole;
    /// Create an index of roles starting from the leafs.
    /// Each node will then contian a "roleset" field indicating whether _any_ descendant has a
    /// given role.
    ///
    /// This will help with other traversal algorithms.
    fn build_rolesets(&mut self);
    /// Build a new tree arena from a pointer-based tree structure.
    fn from_root_node(root: A11yNode) -> Self;
    /// Returns an [`Iterator`] over all leaves in the tree.
    fn iter_leafs(&self) -> impl Iterator<Item = &indextree::Node<Self::Node>> + use<'_, Self>;
    /// Returns a [`ParallelIterator`] over all leaves in the tree.
    fn par_iter_leafs(
        &self,
    ) -> impl ParallelIterator<Item = &indextree::Node<Self::Node>> + use<'_, Self>;
    /// Returns the number of items with a given role.
    fn how_many(&self, role: Role) -> usize;
    /// Returns the number of items with a given role (and avoids subtrees which do not contain the
    /// role).
    fn how_many_roleset(&self, role: Role) -> usize;
    /// Returns the number of items with a given role (and computes this number in parallel).
    fn par_how_many(&self, role: Role) -> usize;
    /// Returns the number of items with a given role (and avoids subtrees which do not contain the
    /// role, and computes in parllel).
    fn par_how_many_roleset(&self, role: Role) -> usize;
    /// Returns the maximum depth of the tree.
    fn max_depth(&self) -> usize;
    /// Returns the maximum depth of the tree (computes in parallel).
    fn par_max_depth(&self) -> usize;
    /// Returns the unique roles in the tree (computed by visiting each node).
    fn unique_roles(&self) -> Vec<Role>;
    /// Returns the unique roles in the tree (computed by visiting each node in parallel).
    fn par_unique_roles(&self) -> Vec<Role>;
    /// Returns the unique roles in the tree (pre-computed).
    fn unique_roles_roleset(&self) -> Vec<Role>;
    /// Returns the first in-order node with a given role.
    fn find_first(&self, role: Role) -> Option<&indextree::Node<Self::Node>>;
    /// Returns the first in-order node with a given role (computes in parallel).
    fn par_find_first(&self, role: Role) -> Option<&indextree::Node<Self::Node>>;
    /// Returns the first in-order node with a given role, ignoring subtrees which do not contain
    /// the role.
    fn find_first_roleset(&self, role: Role) -> Option<&indextree::Node<Self::Node>>;
    /// Returns the first in-order node with a given role, ignoring subtrees which do not contain
    /// the role (computes in parallel).
    fn par_find_first_roleset(&self, role: Role) -> Option<&indextree::Node<Self::Node>>;
    /// Returns the first in-order node with a given role, ignoring subtrees which do not contain
    /// the role (computes using a stack instead of a tree walker).
    fn find_first_stack(&self, role: Role) -> Option<&indextree::Node<Self::Node>>;
    /// Returns number of nodes in the tree.
    fn nodes(&self) -> usize;
}

impl TreeTraversal for TreeCount {
    type Node = NodeCount;
    fn build_rolesets(&mut self) {
        for leaf_id in self.root.descendants(&self.inner).collect::<Vec<_>>() {
            let leaf_roleset = {
                let leaf = self
                    .inner
                    .get_mut(leaf_id)
                    .expect("Valid leaf node")
                    .get_mut();
                leaf.roleset.add(leaf.role);
                leaf.role
            };
            for anc_id in leaf_id.ancestors(&self.inner).collect::<Vec<_>>() {
                let anc = self
                    .inner
                    .get_mut(anc_id)
                    .expect("Valid ancestor node")
                    .get_mut();
                anc.roleset.add(leaf_roleset);
            }
        }
    }
    fn from_root_node(root_node: A11yNode) -> Self {
        let mut tree: Arena<NodeCount> = Arena::new();
        let root_id = NodeCount::from_a11y_node(root_node, &mut tree);
        TreeCount {
            inner: tree,
            root: root_id,
        }
    }
    fn iter_leafs(&self) -> impl Iterator<Item = &indextree::Node<Self::Node>> + use<'_> {
        self.root.descendants(&self.inner).filter_map(|node_id| {
            if node_id.children(&self.inner).next().is_none() {
                self.inner.get(node_id)
            } else {
                None
            }
        })
    }
    fn par_iter_leafs(
        &self,
    ) -> impl ParallelIterator<Item = &indextree::Node<Self::Node>> + use<'_> {
        self.inner.par_iter().filter_map(|node| {
            if node.first_child().is_none() {
                Some(node)
            } else {
                None
            }
        })
    }
    fn how_many(&self, role: Role) -> usize {
        self.root
            .descendants(&self.inner)
            .filter_map(move |node_id| self.inner.get(node_id))
            .filter(|node| node.get().role == role)
            .count()
    }
    fn how_many_roleset(&self, role: Role) -> usize {
        NodeIdExt::descendants_role(self.root, &self.inner, role.into())
            .filter(move |node_id| match self.inner.get(*node_id) {
                None => false,
                Some(node) => node.get().role == role,
            })
            .count()
    }
    fn par_how_many_roleset(&self, role: Role) -> usize {
        let rs: RoleSet = role.into();
        walk_tree(self.root, move |node_id| {
            // children which have no descendants with a given role are ignored
            node_id.children(&self.inner).filter(move |child| {
                self.inner
                    .get(*child)
                    .expect("Valid child")
                    .get()
                    .roleset
                    .contains(rs)
            })
        })
        .filter(move |node_id| self.inner.get(*node_id).expect("Valid index").get().role == role)
        .count()
    }
    fn par_how_many(&self, role: Role) -> usize {
        self.inner
            .par_iter()
            .filter(move |node| node.get().role == role)
            .count()
    }
    fn max_depth(&self) -> usize {
        self.root
            .descendants(&self.inner)
            .map(|item| item.ancestors(&self.inner).count())
            .max()
            .expect("A valid ancestors size!")
    }
    fn par_max_depth(&self) -> usize {
        self.inner
            .par_iter()
            .map(|node| match node.parent() {
                Some(parent) => parent.ancestors(&self.inner).count(),
                None => 0,
            })
            .max()
            .expect("A valid ancestors size!")
            + 1
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
    fn par_unique_roles(&self) -> Vec<Role> {
        self.inner
            .par_iter()
            .map(|node| node.get().role)
            // parllel fold; one Vec per core
            .fold(Vec::new, |mut roles, role| {
                if !roles.contains(&role) {
                    roles.push(role);
                }
                roles
            })
            // Vec<Vec<Role>> -> Iterator<Role>
            .flatten_iter()
            // Iterator<Role> -> Vec<Role>
            .collect::<Vec<Role>>()
            // take the value, sort in in place, deduplicate it in place;
            // then, return it
            .tap_mut(|vec| {
                vec.par_sort_unstable_by(|r1, r2| (*r1 as u32).cmp(&(*r2 as u32)));
                vec.dedup();
            })
    }
    fn unique_roles_roleset(&self) -> Vec<Role> {
        self.inner
            .get(self.root)
            .expect("Root is valid ID!")
            .get()
            .roleset
            .1
            .role_iter()
            .collect()
    }
    fn find_first(&self, role: Role) -> Option<&indextree::Node<NodeCount>> {
        self.root.descendants(&self.inner).find_map(move |node_id| {
            self.inner
                .get(node_id)
                .filter(|&node| node.get().role == role)
        })
    }
    fn par_find_first(&self, role: Role) -> Option<&indextree::Node<NodeCount>> {
        self.inner
            .par_iter()
            .by_exponential_blocks()
            .find_first(|node| node.get().role == role)
    }
    fn find_first_roleset(&self, role: Role) -> Option<&indextree::Node<NodeCount>> {
        NodeIdExt::descendants_role(self.root, &self.inner, role.into()).find_map(move |node_id| {
            self.inner
                .get(node_id)
                .filter(|&node| node.get().role == role)
        })
    }
    fn par_find_first_roleset(&self, role: Role) -> Option<&indextree::Node<NodeCount>> {
        let rs: RoleSet = role.into();
        walk_tree(self.root, move |node_id| {
            // children which have no descendants with a given role are ignored
            node_id.children(&self.inner).filter(move |child| {
                self.inner
                    .get(*child)
                    .expect("Valid child")
                    .get()
                    .roleset
                    .contains(rs)
            })
        })
        .map(move |node_id| self.inner.get(node_id).expect("Valid ID!"))
        .find_first(|node| node.get().role == role)
    }
    fn find_first_stack(&self, role: Role) -> Option<&indextree::Node<Self::Node>> {
        let roles: RoleSet = role.into();
        let mut stack = VecDeque::new();
        stack.reserve(33);
        stack.push_back(self.root);
        while let Some(id) = stack.pop_front() {
            let node = self.inner.get(id).expect("Valid ID!");
            if node.get().role == role {
                return Some(node);
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
    fn nodes(&self) -> usize {
        self.inner.count()
    }
}

/// A tree containing both a role, a roleset for all descendants, and the count of how many roles
/// are in the descendants.
#[derive(Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct TreeCount {
    inner: Arena<NodeCount>,
    root: NodeId,
}

/// A node containing both a role, and a roleset for all descendants.
#[derive(Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct Node {
    /// Role of node.
    role: Role,
    /// Roleset of all descendants.
    roleset: RoleSet,
}
impl HasRole for Node {
    fn roleset(&self) -> RoleSet {
        self.roleset
    }
}
impl Node {
    /// Adds the created [`Node`] to a given arena; returns its new [`NodeId`].
    pub fn from_a11y_node(node: A11yNode, tree: &mut Arena<Node>) -> NodeId {
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

/// An arena-based tree, using [`Node`] as its inner node type.
#[derive(Debug, Deserialize, Serialize, Eq, PartialEq)]
pub struct Tree {
    /// An arena containing all [`Node`]s.
    inner: Arena<Node>,
    /// The [`NodeId`] for the root node.
    root: NodeId,
}
impl TreeTraversal for Tree {
    type Node = Node;
    fn build_rolesets(&mut self) {
        for leaf_id in self.root.descendants(&self.inner).collect::<Vec<_>>() {
            let leaf_roleset = {
                let leaf = self
                    .inner
                    .get_mut(leaf_id)
                    .expect("Valid leaf node")
                    .get_mut();
                leaf.roleset |= leaf.role;
                leaf.roleset
            };
            for anc_id in leaf_id.ancestors(&self.inner).collect::<Vec<_>>() {
                let anc = self
                    .inner
                    .get_mut(anc_id)
                    .expect("Valid ancestor node")
                    .get_mut();
                anc.roleset |= leaf_roleset;
            }
        }
    }
    fn from_root_node(root_node: A11yNode) -> Self {
        let mut tree: Arena<Node> = Arena::new();
        let root_id = Node::from_a11y_node(root_node, &mut tree);
        Tree {
            inner: tree,
            root: root_id,
        }
    }
    fn iter_leafs(&self) -> impl Iterator<Item = &indextree::Node<Node>> + use<'_> {
        self.root.descendants(&self.inner).filter_map(|node_id| {
            if node_id.children(&self.inner).next().is_none() {
                self.inner.get(node_id)
            } else {
                None
            }
        })
    }
    fn par_iter_leafs(&self) -> impl ParallelIterator<Item = &indextree::Node<Node>> + use<'_> {
        self.inner
            .par_iter()
            .filter(|node| node.first_child().is_none())
    }
    fn nodes(&self) -> usize {
        self.inner.count()
    }
    fn find_first(&self, role: Role) -> Option<&indextree::Node<Node>> {
        self.root.descendants(&self.inner).find_map(move |node_id| {
            self.inner
                .get(node_id)
                .filter(|&node| node.get().role == role)
        })
    }
    fn par_find_first(&self, role: Role) -> Option<&indextree::Node<Node>> {
        self.inner
            .par_iter()
            // instead of evenly dividing the task, exponentially increate the offset
            // this finds earlier items sooner
            .by_exponential_blocks()
            .find_first(|node| node.get().role == role)
    }
    fn find_first_roleset(&self, role: Role) -> Option<&indextree::Node<Node>> {
        NodeIdExt::descendants_role(self.root, &self.inner, role.into()).find_map(move |node_id| {
            self.inner
                .get(node_id)
                .filter(|&node| node.get().role == role)
        })
    }
    fn par_find_first_roleset(&self, role: Role) -> Option<&indextree::Node<Node>> {
        let rs: RoleSet = role.into();
        walk_tree(self.root, move |node_id| {
            // children which have no descendants with a given role are ignored
            node_id.children(&self.inner).filter(move |child| {
                self.inner
                    .get(*child)
                    .expect("Valid child")
                    .get()
                    .roleset
                    .contains(rs)
            })
        })
        .map(move |node_id| self.inner.get(node_id).expect("Valid ID!"))
        .find_first(|node| node.get().role == role)
    }
    fn find_first_stack(&self, role: Role) -> Option<&indextree::Node<Self::Node>> {
        let roles: RoleSet = role.into();
        let mut stack = VecDeque::new();
        stack.reserve(33);
        stack.push_back(self.root);
        while let Some(id) = stack.pop_front() {
            let node = self.inner.get(id).expect("Valid ID!");
            if node.get().role == role {
                return Some(node);
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
    fn par_how_many(&self, role: Role) -> usize {
        self.inner
            .par_iter()
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
    fn par_max_depth(&self) -> usize {
        self.inner
            .par_iter()
            .map(|node| match node.parent() {
                Some(parent) => parent.ancestors(&self.inner).count(),
                None => 0,
            })
            .max()
            .expect("A valid ancestors size!")
            + 1
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
    fn par_unique_roles(&self) -> Vec<Role> {
        self.inner
            .par_iter()
            .map(|node| node.get().role)
            // parllel fold; one Vec per core
            .fold(Vec::new, |mut roles, role| {
                if !roles.contains(&role) {
                    roles.push(role);
                }
                roles
            })
            // Vec<Vec<Role>> -> Iterator<Role>
            .flatten_iter()
            // Iterator<Role> -> Vec<Role>
            .collect::<Vec<Role>>()
            // take the value, sort in in place, deduplicate it in place;
            // then, return it
            .tap_mut(|vec| {
                vec.par_sort_unstable_by(|r1, r2| (*r1 as u32).cmp(&(*r2 as u32)));
                vec.dedup();
            })
    }
    fn unique_roles_roleset(&self) -> Vec<Role> {
        self.inner
            .get(self.root)
            .expect("Root is valid ID!")
            .get()
            .roleset
            .role_iter()
            .collect()
    }
    fn how_many_roleset(&self, role: Role) -> usize {
        NodeIdExt::descendants_role(self.root, &self.inner, role.into())
            .filter(move |node_id| match self.inner.get(*node_id) {
                None => false,
                Some(node) => node.get().role == role,
            })
            .count()
    }
    fn par_how_many_roleset(&self, role: Role) -> usize {
        let rs: RoleSet = role.into();
        walk_tree(self.root, move |node_id| {
            // children which have no descendants with a given role are ignored
            node_id.children(&self.inner).filter(move |child| {
                self.inner
                    .get(*child)
                    .expect("Valid child")
                    .get()
                    .roleset
                    .contains(rs)
            })
        })
        .filter(move |node_id| self.inner.get(*node_id).expect("Valid index").get().role == role)
        .count()
    }
}

/// A node in a tree. The standard type which [`Tree`] and [`TreeCount`] use to create their
/// arena-based trees.
///
/// TODO: should also be tested in benchmarks for comparison.
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct A11yNode {
    /// The role of the node.
    role: Role,
    /// The children of the node.
    children: Vec<A11yNode>,
}

#[derive(Clone, Copy)]
struct CharSet {
    pub horizontal: char,
    pub vertical: char,
    pub connector: char,
    pub end_connector: char,
}
/// Defenition of formatting characters for pretty-printing [`A11yNode`].
const SINGLE_LINE: CharSet = CharSet {
    horizontal: '─',
    vertical: '│',
    connector: '├',
    end_connector: '└',
};

impl Display for A11yNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmt_with(f, SINGLE_LINE, &mut Vec::new())
    }
}

impl A11yNode {
    // False positive from clippy
    #[allow(unused_variables)]
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
            }
            leafs += 1;
            prefix.pop();
        }
        Ok(())
    }
}
