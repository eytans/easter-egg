use std::fmt::Debug;
use std::iter::ExactSizeIterator;
use indexmap::IndexMap;

use crate::{ColorId, Id, Language};

/// An equivalence class of enodes.
#[non_exhaustive]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EClass<L, D> {
    /// This eclass's id.
    pub id: Id,
    /// The equivalent enodes in this equivalence class with their associated colors.
    /// No color set means it is a black edge (which will never be removed).
    pub nodes: Vec<L>,
    /// The analysis data associated with this eclass.
    pub data: D,
    pub(crate) parents: Vec<(L, Id)>,
    pub(crate) changed_parents: Vec<(L, Id)>,

    /// Each EClass has a unique color (None is black, i.e. default).
    #[cfg(feature = "colored")]
    pub(crate) color: Option<ColorId>,
    /// Colored parents are colored_canonized pointing to the black ID of the class.
    #[cfg(feature = "colored")]
    pub(crate) colored_parents: IndexMap<ColorId, Vec<(L, Id)>>,
    #[cfg(feature = "colored")]
    pub(crate) colord_changed_parents: IndexMap<ColorId, Vec<(L, Id)>>,
}

impl<L, D> EClass<L, D> {
    /// Returns `true` if the `eclass` is empty.
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Returns the number of enodes in this eclass.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Iterates over the enodes in this eclass.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &L> {
        self.nodes.iter()
    }

    /// Returns the color of the EClass if exists. None means it's a black class.
    pub fn color(&self) -> Option<ColorId> {
        self.color
    }
}

impl<L: Language, D> EClass<L, D> {
    /// Iterates over the childless enodes in this eclass.
    pub fn leaves(&self) -> impl Iterator<Item = &L> {
        self.nodes.iter().filter(|n| n.is_leaf())
    }

    /// Asserts that the childless enodes in this eclass are unique.
    pub fn assert_unique_leaves(&self)
    where
        L: Language,
    {
        let mut leaves = self.leaves();
        if let Some(first) = leaves.next() {
            assert!(
                leaves.all(|l| l == first),
                "Different leaves in eclass {}: {:?}",
                self.id,
                self.leaves().collect::<indexmap::IndexSet<_>>()
            );
        }
    }
}
