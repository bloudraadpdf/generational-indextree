//! Node.

#[cfg(not(feature = "std"))]
use core::fmt;

#[cfg(feature = "deser")]
use serde::{Deserialize, Serialize};

#[cfg(feature = "std")]
use std::fmt;

#[cfg(not(feature = "std"))]
use core::{convert::TryFrom, num::NonZeroU32};
#[cfg(feature = "std")]
use std::{convert::TryFrom, num::NonZeroU32};

use crate::NodeId;

/// The slot of a linked node. A link names the live node of its slot: a removal detaches the node from its
/// relatives first.
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "deser", derive(Deserialize, Serialize))]
pub(crate) struct Link(NonZeroU32);

impl Link {
    pub(crate) fn of(id: NodeId) -> Self {
        let slot = u32::try_from(id.slot()).expect("an arena has fewer than 2^32 slots");
        Self(NonZeroU32::new(slot + 1).expect("a slot below 2^32 - 1 has a link"))
    }

    pub(crate) fn slot(self) -> usize {
        (self.0.get() - 1) as usize
    }
}

impl fmt::Display for Link {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.slot())
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
#[cfg_attr(feature = "deser", derive(Deserialize, Serialize))]
/// A node within a particular `Arena`.
pub struct Node<T> {
    // Keep these private (with read-only accessors) so that we can keep them
    // consistent. E.g. the parent of a node’s child is that node.
    pub(crate) parent: Option<Link>,
    pub(crate) previous_sibling: Option<Link>,
    pub(crate) next_sibling: Option<Link>,
    pub(crate) first_child: Option<Link>,
    pub(crate) last_child: Option<Link>,
    /// The actual data which will be stored within the tree.
    pub(crate) data: T,
}

impl<T> Node<T> {
    /// Returns a reference to the node data.
    pub fn get(&self) -> &T {
        &self.data
    }

    /// Returns a mutable reference to the node data.
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.data
    }

    /// Creates a new `Node` with the default state and the given data.
    pub(crate) fn new(data: T) -> Self {
        Self {
            parent: None,
            previous_sibling: None,
            next_sibling: None,
            first_child: None,
            last_child: None,
            data,
        }
    }

    /// Checks if the node is detached.
    pub(crate) fn is_detached(&self) -> bool {
        self.parent.is_none() && self.previous_sibling.is_none() && self.next_sibling.is_none()
    }
}

impl<T> fmt::Display for Node<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(parent) = self.parent {
            write!(f, "parent: {}; ", parent)?;
        } else {
            write!(f, "no parent; ")?;
        }
        if let Some(previous_sibling) = self.previous_sibling {
            write!(f, "previous sibling: {}; ", previous_sibling)?;
        } else {
            write!(f, "no previous sibling; ")?;
        }
        if let Some(next_sibling) = self.next_sibling {
            write!(f, "next sibling: {}; ", next_sibling)?;
        } else {
            write!(f, "no next sibling; ")?;
        }
        if let Some(first_child) = self.first_child {
            write!(f, "first child: {}; ", first_child)?;
        } else {
            write!(f, "no first child; ")?;
        }
        if let Some(last_child) = self.last_child {
            write!(f, "last child: {}; ", last_child)?;
        } else {
            write!(f, "no last child; ")?;
        }
        Ok(())
    }
}
