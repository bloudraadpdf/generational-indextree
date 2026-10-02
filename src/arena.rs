//! Arena.

#[cfg(not(feature = "std"))]
use core::ops::{Index, IndexMut};
#[cfg(feature = "std")]
use std::ops::{Index, IndexMut};

use generational_arena::Arena as GenerationalArena;
#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "par_iter")]
use rayon::prelude::*;

use crate::{Node, NodeId};

#[derive(Clone)]
#[cfg_attr(feature = "deser", derive(Deserialize, Serialize))]
/// An `Arena` structure containing certain [`Node`]s.
///
/// [`Node`]: struct.Node.html
pub struct Arena<T> {
    pub(crate) nodes: GenerationalArena<Node<T>>,
}

impl<T> Arena<T> {
    /// Creates a new empty `Arena`.
    pub fn new() -> Arena<T> {
        Self::default()
    }

    /// Create a new empty `Arena` with pre-allocated memory for `n` items.
    pub fn with_capacity(n: usize) -> Arena<T> {
        Self {
            nodes: GenerationalArena::with_capacity(n),
        }
    }

    /// Returns the number of node slots of the arena: the nodes and the free
    /// slots that a new node can take without an allocation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use generational_indextree::Arena;
    /// let arena: Arena<&str> = Arena::with_capacity(10);
    ///
    /// assert_eq!(arena.capacity(), 10);
    /// ```
    pub fn capacity(&self) -> usize {
        self.nodes.capacity()
    }

    /// Returns the ID of the node at slot `index`, if a node occupies the slot.
    ///
    /// # Examples
    ///
    /// ```
    /// # use generational_indextree::Arena;
    /// let mut arena = Arena::new();
    /// let foo = arena.new_node("foo");
    /// assert_eq!(arena.node_id_at(0), Some(foo));
    /// assert_eq!(arena.node_id_at(1), None);
    /// ```
    pub fn node_id_at(&self, index: usize) -> Option<NodeId> {
        self.nodes
            .get_unknown_gen(index)
            .map(|(_, index)| NodeId::from_index(index))
    }

    /// Shrinks the capacity of the arena to its last node.
    ///
    /// The free slots after the last node go, so the arena keeps no memory
    /// for them. Each node ID stays valid.
    ///
    /// # Examples
    ///
    /// ```
    /// # use generational_indextree::Arena;
    /// let mut arena = Arena::with_capacity(10);
    /// let foo = arena.new_node("foo");
    /// arena.shrink_to_fit();
    ///
    /// assert_eq!(*arena[foo].get(), "foo");
    /// ```
    pub fn shrink_to_fit(&mut self) {
        self.nodes.shrink_to_fit();
    }

    /// Creates a new node from its associated data.
    ///
    /// # Panics
    ///
    /// Panics if the arena already has `usize::max_value()` nodes.
    ///
    /// # Examples
    ///
    /// ```
    /// # use generational_indextree::Arena;
    /// let mut arena = Arena::new();
    /// let foo = arena.new_node("foo");
    ///
    /// assert_eq!(*arena[foo].get(), "foo");
    /// ```
    pub fn new_node(&mut self, data: T) -> NodeId {
        NodeId::from_index(self.nodes.insert(Node::new(data)))
    }

    /// Creates a new node via specified `create` function.
    ///
    /// `create` is called with the new node's node ID, allowing nodes that know their own ID.
    ///
    /// # Panics
    ///
    /// Panics if the arena already has `usize::max_value()` nodes.
    ///
    /// # Examples
    ///
    /// ```
    /// # use generational_indextree::{Arena, NodeId};
    /// let mut arena = Arena::new();
    /// struct A { id: NodeId, val: u32 }
    /// let foo = arena.new_node_with(|id| A { id, val: 10 });
    ///
    /// assert_eq!(arena[foo].get().val, 10);
    /// assert_eq!(arena[foo].get().id, foo);
    /// ```
    pub fn new_node_with(&mut self, create: impl FnOnce(NodeId) -> T) -> NodeId {
        NodeId::from_index(
            self.nodes
                .insert_with(|idx| Node::new(create(NodeId::from_index(idx)))),
        )
    }

    /// Counts the number of nodes in arena and returns it.
    ///
    /// # Examples
    ///
    /// ```
    /// # use generational_indextree::Arena;
    /// let mut arena = Arena::new();
    /// let foo = arena.new_node("foo");
    /// let _bar = arena.new_node("bar");
    /// assert_eq!(arena.count(), 2);
    ///
    /// foo.remove(&mut arena);
    /// assert_eq!(arena.count(), 1);
    /// ```
    pub fn count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns `true` if arena has no nodes, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # use generational_indextree::Arena;
    /// let mut arena = Arena::new();
    /// assert!(arena.is_empty());
    ///
    /// let foo = arena.new_node("foo");
    /// assert!(!arena.is_empty());
    ///
    /// foo.remove(&mut arena);
    /// assert!(arena.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }

    /// Returns a reference to the node with the given id if in the arena.
    ///
    /// Returns `None` if not available.
    ///
    /// # Examples
    ///
    /// ```
    /// # use generational_indextree::{Arena, NodeId};
    /// let mut arena = Arena::new();
    /// let foo = arena.new_node("foo");
    /// assert_eq!(arena.get(foo).map(|node| *node.get()), Some("foo"));
    /// ```
    ///
    /// Note that this does not check whether the given node ID is created by
    /// the arena.
    ///
    /// ```
    /// # use generational_indextree::Arena;
    /// let mut arena = Arena::new();
    /// let foo = arena.new_node("foo");
    /// let bar = arena.new_node("bar");
    /// assert_eq!(arena.get(foo).map(|node| *node.get()), Some("foo"));
    ///
    /// let mut another_arena = Arena::new();
    /// let _ = another_arena.new_node("Another arena");
    /// assert_eq!(another_arena.get(foo).map(|node| *node.get()), Some("Another arena"));
    /// assert!(another_arena.get(bar).is_none());
    /// ```
    pub fn get(&self, id: NodeId) -> Option<&Node<T>> {
        self.nodes.get(id.get_index())
    }

    /// Returns a mutable reference to the node with the given id if in the
    /// arena.
    ///
    /// Returns `None` if not available.
    ///
    /// # Examples
    ///
    /// ```
    /// # use generational_indextree::{Arena, NodeId};
    /// let mut arena = Arena::new();
    /// let foo = arena.new_node("foo");
    /// assert_eq!(arena.get(foo).map(|node| *node.get()), Some("foo"));
    ///
    /// *arena.get_mut(foo).expect("The `foo` node exists").get_mut() = "FOO!";
    /// assert_eq!(arena.get(foo).map(|node| *node.get()), Some("FOO!"));
    /// ```
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut Node<T>> {
        self.nodes.get_mut(id.get_index())
    }

    /// Get a pair of exclusive references to the elements at index `i1` and `i2` if it is in the
    /// arena.
    ///
    /// If the element at index `i1` or `i2` is not in the arena, then `None` is returned for this
    /// element.
    ///
    /// # Panics
    ///
    /// Panics if `i1` and `i2` are pointing to the same item of the arena.
    ///
    /// # Examples
    ///
    /// ```
    /// use generational_indextree::Arena;
    ///
    /// let mut arena = Arena::new();
    /// let idx1 = arena.new_node("foo");
    /// let idx2 = arena.new_node("bar");
    ///
    /// {
    ///     let (item1, item2) = arena.get2_mut(idx1, idx2);
    ///
    ///     *item1.unwrap().get_mut() = "jig";
    ///     *item2.unwrap().get_mut() = "saw";
    /// }
    ///
    /// assert_eq!(arena[idx1].get(), &"jig");
    /// assert_eq!(arena[idx2].get(), &"saw");
    /// ```
    pub fn get2_mut(
        &mut self,
        i1: NodeId,
        i2: NodeId,
    ) -> (Option<&mut Node<T>>, Option<&mut Node<T>>) {
        self.nodes.get2_mut(i1.get_index(), i2.get_index())
    }

    /// Returns an iterator of all nodes in the arena in storage-order.
    ///
    /// # Examples
    ///
    /// ```
    /// # use generational_indextree::Arena;
    /// let mut arena = Arena::new();
    /// let _foo = arena.new_node("foo");
    /// let _bar = arena.new_node("bar");
    ///
    /// let mut iter = arena.iter();
    /// assert_eq!(iter.next().map(|node| *node.get()), Some("foo"));
    /// assert_eq!(iter.next().map(|node| *node.get()), Some("bar"));
    /// assert_eq!(iter.next().map(|node| *node.get()), None);
    /// ```
    ///
    /// ```
    /// # use generational_indextree::Arena;
    /// let mut arena = Arena::new();
    /// let _foo = arena.new_node("foo");
    /// let bar = arena.new_node("bar");
    /// bar.remove(&mut arena);
    ///
    /// let mut iter = arena.iter();
    /// assert_eq!(iter.next().map(|node| *node.get()), Some("foo"));
    /// assert_eq!(iter.next().map(|node| *node.get()), None);
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = &Node<T>> {
        self.nodes.iter().map(|pair| pair.1)
    }

    /// Returns an iterator of all pairs (NodeId, &Node<T>) in the arena in storage-order.
    ///
    /// ```
    /// # use generational_indextree::Arena;
    /// let mut arena = Arena::new();
    /// let _foo = arena.new_node("foo");
    /// let _bar = arena.new_node("bar");
    ///
    /// let mut iter = arena.iter_pairs();
    /// assert_eq!(iter.next().map(|node| (node.0, *node.1.get())), Some((_foo, "foo")));
    /// assert_eq!(iter.next().map(|node| (node.0, *node.1.get())), Some((_bar, "bar")));
    /// assert_eq!(iter.next().map(|node| (node.0, *node.1.get())), None);
    /// ```
    pub fn iter_pairs(&self) -> impl Iterator<Item = (NodeId, &Node<T>)> {
        self.nodes
            .iter()
            .map(|pair| (NodeId::from_index(pair.0), pair.1))
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Self {
            nodes: GenerationalArena::new(),
        }
    }
}

impl<T: core::fmt::Debug> core::fmt::Debug for Arena<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list()
            .entries(self.nodes.iter().map(|(idx, node)| (idx, &node.data)))
            .finish()
    }
}

impl<T> Index<NodeId> for Arena<T> {
    type Output = Node<T>;

    fn index(&self, node: NodeId) -> &Node<T> {
        &self.nodes[node.get_index()]
    }
}

impl<T> IndexMut<NodeId> for Arena<T> {
    fn index_mut(&mut self, node: NodeId) -> &mut Node<T> {
        &mut self.nodes[node.get_index()]
    }
}

impl<T: PartialEq> PartialEq for Arena<T> {
    fn eq(&self, other: &Self) -> bool {
        let mut equal = self.nodes.len() == other.nodes.len();
        let mut self_iter = self.iter();
        let mut other_iter = other.iter();
        while equal {
            let lhs = self_iter.next();
            let rhs = other_iter.next();
            equal = lhs == rhs;
            if lhs.is_none() {
                break;
            }
        }
        equal
    }
}

impl<T: PartialEq> Eq for Arena<T> {}

#[test]
fn reuse_node() {
    let mut arena = Arena::with_capacity(3);
    let n1_id = arena.new_node("1");
    let n2_id = arena.new_node("2");
    let n3_id = arena.new_node("3");
    n1_id.remove(&mut arena);
    n2_id.remove(&mut arena);
    n3_id.remove(&mut arena);
    let new_n1_id = arena.new_node("1");
    let new_n2_id = arena.new_node("2");
    let new_n3_id = arena.new_node("3");
    assert_eq!(arena.nodes.len(), 3);
    assert_ne!(n1_id, new_n1_id);
    assert_ne!(n2_id, new_n2_id);
    assert_ne!(n3_id, new_n3_id);
}

#[cfg(feature = "std")]
#[test]
fn debug_arena() {
    let mut arena = Arena::with_capacity(10);
    let _n1_id = arena.new_node("1");
    let n2_id = arena.new_node("2");
    let n3_id = arena.new_node("3");
    let n4_id = arena.new_node("4");
    let _n5_id = arena.new_node("5");
    n2_id.remove(&mut arena);
    n3_id.remove(&mut arena);
    n4_id.remove(&mut arena);
    let _n6_id = arena.new_node("6");

    assert_eq!(
        format!("{:?}", arena),
        "[(Index { index: 0, generation: 0 }, \"1\"), (Index { index: 3, generation: 3 }, \"6\"), (Index { index: 4, generation: 0 }, \"5\")]"
    );
}
