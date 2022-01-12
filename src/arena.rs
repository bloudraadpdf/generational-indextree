//! Arena.

#[cfg(not(feature = "std"))]
use core::{
    num::NonZeroUsize,
    ops::{Index, IndexMut},
};
#[cfg(feature = "std")]
use std::ops::{Index, IndexMut};

use slotmap::{Key, SlotMap};
#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "par_iter")]
use rayon::prelude::*;

use crate::{Node, NodeId};

#[derive(Clone, Debug)]
#[cfg_attr(feature = "deser", derive(Deserialize, Serialize))]
/// An `Arena` structure containing certain [`Node`]s.
///
/// [`Node`]: struct.Node.html
pub struct Arena<K: Key, T> {
    pub(crate) nodes: SlotMap<K,Node<K, T>>,
}

impl<K: Key, T> Arena<K, T> {
    /// Creates a new empty `Arena`.
    pub fn new() -> Arena<K, T> {
        Self::default()
    }

    /// Create a new empty `Arena` with pre-allocated memory for `n` items.
    pub fn with_capacity(n: usize) -> Arena<K, T> {
        Self {
            nodes: SlotMap::with_capacity_and_key(n),
        }
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
    /// let mut arena = Arena::<slotmap::DefaultKey,_>::new();
    /// let foo = arena.new_node("foo");
    ///
    /// assert_eq!(*arena[foo].get(), "foo");
    /// ```
    pub fn new_node(&mut self, data: T) -> NodeId<K> {
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
    /// let mut arena = Arena::<slotmap::DefaultKey,_>::new();
    /// struct A { id: NodeId<slotmap::DefaultKey>, val: u32 }
    /// let foo = arena.new_node_with(|id| A { id, val: 10 });
    ///
    /// assert_eq!(arena[foo].get().val, 10);
    /// assert_eq!(arena[foo].get().id, foo);
    /// ```
    pub fn new_node_with(&mut self, create: impl FnOnce(NodeId<K>) -> T) -> NodeId<K> {
        NodeId::from_index(
            self.nodes
                .insert_with_key(|idx| Node::new(create(NodeId::from_index(idx)))),
        )
    }

    /// Counts the number of nodes in arena and returns it.
    ///
    /// # Examples
    ///
    /// ```
    /// # use generational_indextree::Arena;
    /// let mut arena = Arena::<slotmap::DefaultKey,_>::new();
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
    /// let mut arena = Arena::<slotmap::DefaultKey,_>::new();
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
    /// let mut arena = Arena::<slotmap::DefaultKey,_>::new();
    /// let foo = arena.new_node("foo");
    /// assert_eq!(arena.get(foo).map(|node| *node.get()), Some("foo"));
    /// ```
    ///
    /// Note that this does not check whether the given node ID is created by
    /// the arena.
    ///
    /// ```
    /// # use generational_indextree::Arena;
    /// let mut arena = Arena::<slotmap::DefaultKey,_>::new();
    /// let foo = arena.new_node("foo");
    /// let bar = arena.new_node("bar");
    /// assert_eq!(arena.get(foo).map(|node| *node.get()), Some("foo"));
    ///
    /// let mut another_arena = Arena::<slotmap::DefaultKey,_>::new();
    /// let _ = another_arena.new_node("Another arena");
    /// assert_eq!(another_arena.get(foo).map(|node| *node.get()), Some("Another arena"));
    /// assert!(another_arena.get(bar).is_none());
    /// ```
    pub fn get(&self, id: NodeId<K>) -> Option<&Node<K, T>> {
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
    /// let mut arena = Arena::<slotmap::DefaultKey,_>::new();
    /// let foo = arena.new_node("foo");
    /// assert_eq!(arena.get(foo).map(|node| *node.get()), Some("foo"));
    ///
    /// *arena.get_mut(foo).expect("The `foo` node exists").get_mut() = "FOO!";
    /// assert_eq!(arena.get(foo).map(|node| *node.get()), Some("FOO!"));
    /// ```
    pub fn get_mut(&mut self, id: NodeId<K>) -> Option<&mut Node<K, T>> {
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
    /// let mut arena = Arena::<slotmap::DefaultKey,_>::new();
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
        i1: NodeId<K>,
        i2: NodeId<K>,
    ) -> (Option<&mut Node<K, T>>, Option<&mut Node<K, T>>) {
        self.nodes.get_disjoint_mut([i1.get_index(), i2.get_index()])
            .map(|pair| {
                let [zero,one] = pair;
                (Some(zero),Some(one))
            })
            .unwrap_or((None,None))
    }

    /// Returns an iterator of all nodes in the arena in storage-order.
    ///
    /// # Examples
    ///
    /// ```
    /// # use generational_indextree::Arena;
    /// let mut arena = Arena::<slotmap::DefaultKey,_>::new();
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
    /// let mut arena = Arena::<slotmap::DefaultKey,_>::new();
    /// let _foo = arena.new_node("foo");
    /// let bar = arena.new_node("bar");
    /// bar.remove(&mut arena);
    ///
    /// let mut iter = arena.iter();
    /// assert_eq!(iter.next().map(|node| *node.get()), Some("foo"));
    /// assert_eq!(iter.next().map(|node| *node.get()), None);
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = &Node<K, T>> {
        self.nodes.iter().map(|pair| pair.1)
    }

    /// Returns an iterator of all pairs (NodeId<K>, &Node<K, T>) in the arena in storage-order.
    ///
    /// ```
    /// # use generational_indextree::Arena;
    /// let mut arena = Arena::<slotmap::DefaultKey,_>::new();
    /// let _foo = arena.new_node("foo");
    /// let _bar = arena.new_node("bar");
    ///
    /// let mut iter = arena.iter_pairs();
    /// assert_eq!(iter.next().map(|node| (node.0, *node.1.get())), Some((_foo, "foo")));
    /// assert_eq!(iter.next().map(|node| (node.0, *node.1.get())), Some((_bar, "bar")));
    /// assert_eq!(iter.next().map(|node| (node.0, *node.1.get())), None);
    /// ```
    pub fn iter_pairs(&self) -> impl Iterator<Item = (NodeId<K>, &Node<K, T>)> {
        self.nodes
            .iter()
            .map(|pair| (NodeId::from_index(pair.0), pair.1))
    }
}

impl<K: Key, T> Default for Arena<K, T> {
    fn default() -> Self {
        Self {
            nodes: SlotMap::with_capacity_and_key(0),
        }
    }
}

impl<K: Key, T> Index<NodeId<K>> for Arena<K, T> {
    type Output = Node<K,T>;

    fn index(&self, node: NodeId<K>) -> &Node<K, T> {
        &self.nodes[node.get_index()]
    }
}

impl<K: Key, T> IndexMut<NodeId<K>> for Arena<K, T> {
    fn index_mut(&mut self, node: NodeId<K>) -> &mut Node<K, T> {
        &mut self.nodes[node.get_index()]
    }
}

impl<K: Key, T: PartialEq> PartialEq for Arena<K, T> {
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

impl<K: Key, T: PartialEq> Eq for Arena<K, T> {}

#[test]
fn reuse_node() {
    let mut arena = Arena::<slotmap::DefaultKey,_>::with_capacity(3);
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
