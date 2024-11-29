use crate::RoleSet;
use indextree::{
	NodeId, NodeEdge, Arena
};

pub trait NodeIdExt {
	fn descendants_role<T>(self, arena: &Arena<T>, role: RoleSet) -> DescendantsRole<'_, T>;
  fn traverse_role<T>(self, arena: &Arena<T>, role: RoleSet) -> TraverseRole<'_, T>;
  fn search_role<T>(self, arena: &Arena<T>, role: RoleSet) -> SearchRole<'_, T>;
}

impl NodeIdExt for NodeId {
    fn descendants_role<T>(self, arena: &Arena<T>, role: RoleSet) -> DescendantsRole<'_, T> {
        DescendantsRole::new(arena, self, role)
    }
    fn traverse_role<T>(self, arena: &Arena<T>, role: RoleSet) -> TraverseRole<'_, T> {
        TraverseRole::new(arena, self, role)
    }
    fn search_role<T>(self, arena: &Arena<T>, role: RoleSet) -> SearchRole<'_, T> {
        SearchRole::new(arena, self, role)
    }
}
pub struct DescendantsRole<'a, T>(TraverseRole<'a, T>);

impl<'a, T> DescendantsRole<'a, T> {
    pub(crate) fn new(arena: &'a Arena<T>, current: NodeId, role: RoleSet) -> Self {
        Self(TraverseRole::new(arena, current, role))
    }
}
 
impl<'a, T> Iterator for DescendantsRole<'a, T> 
where T: HasRole {
    type Item = NodeId;

    fn next(&mut self) -> Option<NodeId> {
        self.0.find_map(|edge| match edge {
            NodeEdge::Start(node) => Some(node),
            NodeEdge::End(_) => None,
        })
    }
}
 
impl<'a, T> core::iter::FusedIterator for DescendantsRole<'a, T> 
where T: HasRole {}

trait NodeEdgeExt {
    fn next_traverse_role<T>(self, arena: &Arena<T>, role: RoleSet) -> Option<Self>
			where Self: Sized,
						T: HasRole ;
}
pub trait HasRole {
	fn roleset(&self) -> RoleSet;
}
impl NodeEdgeExt for NodeEdge {
    fn next_traverse_role<T>(self, arena: &Arena<T>, role: RoleSet) -> Option<Self> 
			where Self: Sized,
						T: HasRole {
        match self {
            NodeEdge::Start(node) => match arena[node].first_child() {
                Some(first_child) => {
									if arena[first_child].get().roleset().contains(role) { Some(NodeEdge::Start(first_child)) } else {
											Some(NodeEdge::End(first_child))
									}
								}
                None => Some(NodeEdge::End(node)),
            },
            NodeEdge::End(node) => {
                let node = &arena[node];
                match node.next_sibling() {
                    Some(next_sibling) => {
											if arena[next_sibling].get().roleset().contains(role) { Some(NodeEdge::Start(next_sibling)) } else {
												NodeEdge::End(next_sibling).next_traverse_role(&arena, role)
											}
										},
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
		where T: HasRole {
        if next == NodeEdge::End(self.root) {
            return None;
        }
        next.next_traverse_role(self.arena, self.role)
    }

    /// Returns a reference to the arena.
    #[inline]
    #[must_use]
    pub(crate) fn arena(&self) -> &Arena<T> {
        self.arena
    }
}
pub struct SearchRole<'a, T> {
    arena: &'a Arena<T>,
    root: NodeId,
    next: Option<NodeEdge>,
		role: RoleSet,
}
impl<'a, T> SearchRole<'a, T> {
    pub(crate) fn new(arena: &'a Arena<T>, current: NodeId, role: RoleSet) -> Self {
        Self {
            arena,
            root: current,
            next: Some(NodeEdge::Start(current)),
						role,
        }
    }

    /// Calculates the next node.
    fn next_of_next(&mut self, next: NodeEdge) -> Option<NodeEdge> 
		where T: HasRole {
				if next != NodeEdge::End(self.root) {
					return next.next_traverse_role(self.arena, self.role);
				}
				let node = &self.arena[self.root];
				if let Some(sib) = node.next_sibling() {
					self.root = sib;
					return Some(NodeEdge::Start(sib));
				}
				if let Some(par) = node.parent() {
					self.root = par;
					return Some(NodeEdge::Start(par));
				}
				None
    }

    /// Returns a reference to the arena.
    #[inline]
    #[must_use]
    pub(crate) fn arena(&self) -> &Arena<T> {
        self.arena
    }
}

impl<'a, T> Iterator for TraverseRole<'a, T> 
where T: HasRole {
    type Item = NodeEdge;

    fn next(&mut self) -> Option<NodeEdge> {
        let next = self.next.take()?;
        self.next = self.next_of_next(next);
        Some(next)
    }
}
impl<'a, T> core::iter::FusedIterator for TraverseRole<'a, T> 
where T: HasRole {}
