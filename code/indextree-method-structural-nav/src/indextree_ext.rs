use crate::RoleSet;
use indextree::{Arena, NodeEdge, NodeId};

/// Take a [`NodeId`] and traverse it using a custom iterator.
/// Only needed sequentially, since `rayon` provides [`rayon::iter::walk_tree`] which gives similar
/// functionality in parallel.
pub trait NodeIdExt {
    /// Traverse descendants, ignoring subtrees whose roleset does not contain the given roleset.
    fn descendants_role<T>(self, arena: &Arena<T>, role: RoleSet) -> DescendantsRole<'_, T>;
    /// Traverse all nodes descendants first, then next siblings, then parent's next siblings, etc.
    /// Ignoring all subtrees whose roleset does not contain the given roleset.
    fn traverse_role<T>(self, arena: &Arena<T>, role: RoleSet) -> TraverseRole<'_, T>;
}

impl NodeIdExt for NodeId {
    fn descendants_role<T>(self, arena: &Arena<T>, role: RoleSet) -> DescendantsRole<'_, T> {
        DescendantsRole::new(arena, self, role)
    }
    fn traverse_role<T>(self, arena: &Arena<T>, role: RoleSet) -> TraverseRole<'_, T> {
        TraverseRole::new(arena, self, role)
    }
}
pub struct DescendantsRole<'a, T>(TraverseRole<'a, T>);

impl<'a, T> DescendantsRole<'a, T> {
    pub(crate) fn new(arena: &'a Arena<T>, current: NodeId, role: RoleSet) -> Self {
        Self(TraverseRole::new(arena, current, role))
    }
}

impl<T> Iterator for DescendantsRole<'_, T>
where
    T: HasRole,
{
    type Item = NodeId;

    fn next(&mut self) -> Option<NodeId> {
        self.0.find_map(|edge| match edge {
            NodeEdge::Start(node) => Some(node),
            NodeEdge::End(_) => None,
        })
    }
}

impl<T> core::iter::FusedIterator for DescendantsRole<'_, T> where T: HasRole {}

trait NodeEdgeExt {
    fn next_traverse_role<T>(self, arena: &Arena<T>, role: RoleSet) -> Option<Self>
    where
        Self: Sized,
        T: HasRole;
}
/// Indication that a type contains a [`RoleSet`].
/// All inner [`crate::TreeTraversal::Node`] types must implement this so that the `RoleSet` can be
/// accessed generically.
pub trait HasRole {
    /// Get the inner [`RoleSet`].
    fn roleset(&self) -> RoleSet;
}
impl NodeEdgeExt for NodeEdge {
    fn next_traverse_role<T>(self, arena: &Arena<T>, role: RoleSet) -> Option<Self>
    where
        Self: Sized,
        T: HasRole,
    {
        match self {
            NodeEdge::Start(node) => match arena[node].first_child() {
                Some(first_child) => {
                    if arena[first_child].get().roleset().contains(role) {
                        Some(NodeEdge::Start(first_child))
                    } else {
                        Some(NodeEdge::End(first_child))
                    }
                }
                None => Some(NodeEdge::End(node)),
            },
            NodeEdge::End(node) => {
                let node = &arena[node];
                match node.next_sibling() {
                    Some(next_sibling) => {
                        if arena[next_sibling].get().roleset().contains(role) {
                            Some(NodeEdge::Start(next_sibling))
                        } else {
                            NodeEdge::End(next_sibling).next_traverse_role(arena, role)
                        }
                    }
                    // `node.parent()` here can only be `None` if the tree has
                    // been modified during iteration, but silently stoping
                    // iteration seems a more sensible behavior than panicking.
                    None => node.parent().map(NodeEdge::End),
                }
            }
        }
    }
}
pub struct TraverseRole<'a, T> {
    arena: &'a Arena<T>,
    root: NodeId,
    next: Option<NodeEdge>,
    role: RoleSet,
}
impl<'a, T> TraverseRole<'a, T> {
    pub(crate) fn new(arena: &'a Arena<T>, current: NodeId, role: RoleSet) -> Self {
        Self {
            arena,
            root: current,
            next: Some(NodeEdge::Start(current)),
            role,
        }
    }

    /// Calculates the next node.
    fn next_of_next(&self, next: NodeEdge) -> Option<NodeEdge>
    where
        T: HasRole,
    {
        if next == NodeEdge::End(self.root) {
            return None;
        }
        next.next_traverse_role(self.arena, self.role)
    }
}

impl<T> Iterator for TraverseRole<'_, T>
where
    T: HasRole,
{
    type Item = NodeEdge;

    fn next(&mut self) -> Option<NodeEdge> {
        let next = self.next.take()?;
        self.next = self.next_of_next(next);
        Some(next)
    }
}
impl<T> core::iter::FusedIterator for TraverseRole<'_, T> where T: HasRole {}
