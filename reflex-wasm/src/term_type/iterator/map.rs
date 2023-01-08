// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{DependencyList, Eagerness, GraphNode, Internable, SerializeJson, StackOffset};
use serde_json::Value as JsonValue;

use crate::{
    allocator::Arena,
    hash::{TermHash, TermHasher, TermSize},
    ArenaPointer, ArenaRef, Term,
};
use reflex_macros::PointerIter;

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct MapIteratorTerm {
    pub source: ArenaPointer,
    pub iteratee: ArenaPointer,
}
impl TermSize for MapIteratorTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for MapIteratorTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        hasher.hash(&self.source, arena).hash(&self.iteratee, arena)
    }
}

impl<A: Arena + Clone> ArenaRef<MapIteratorTerm, A> {
    pub fn source(&self) -> ArenaRef<Term, A> {
        ArenaRef::<Term, _>::new(self.arena.clone(), self.read_value(|term| term.source))
    }
    pub fn iteratee(&self) -> ArenaRef<Term, A> {
        ArenaRef::<Term, _>::new(self.arena.clone(), self.read_value(|term| term.iteratee))
    }
}

impl<A: Arena + Clone> SerializeJson for ArenaRef<MapIteratorTerm, A> {
    fn to_json(&self) -> Result<JsonValue, String> {
        Err(format!("Unable to serialize term: {}", self))
    }
    fn patch(&self, target: &Self) -> Result<Option<JsonValue>, String> {
        Err(format!(
            "Unable to create patch for terms: {}, {}",
            self, target
        ))
    }
}

impl<A: Arena + Clone> PartialEq for ArenaRef<MapIteratorTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.source() == other.source() && other.iteratee() == other.iteratee()
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<MapIteratorTerm, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<MapIteratorTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<MapIteratorTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MapIterator")
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<MapIteratorTerm, A> {
    fn size(&self) -> usize {
        1 + self.source().size() + self.iteratee().size()
    }
    fn capture_depth(&self) -> StackOffset {
        self.source()
            .capture_depth()
            .max(self.iteratee().capture_depth())
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        self.source().free_variables()
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        self.source().count_variable_usages(offset)
    }
    fn dynamic_dependencies(&self, deep: bool) -> DependencyList {
        if deep {
            self.source()
                .dynamic_dependencies(deep)
                .into_iter()
                .chain(self.iteratee().dynamic_dependencies(deep))
                .collect()
        } else {
            DependencyList::empty()
        }
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        if deep {
            self.source().has_dynamic_dependencies(deep)
                || self.iteratee().has_dynamic_dependencies(deep)
        } else {
            false
        }
    }
    fn is_static(&self) -> bool {
        true
    }
    fn is_atomic(&self) -> bool {
        self.source().is_atomic() && self.iteratee().is_atomic()
    }
    fn is_complex(&self) -> bool {
        true
    }
}

impl<A: Arena + Clone> Internable for ArenaRef<MapIteratorTerm, A> {
    fn should_intern(&self, _eager: Eagerness) -> bool {
        self.capture_depth() == 0
    }
}
#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn map_iterator() {
        assert_eq!(
            TermType::MapIterator(MapIteratorTerm {
                source: ArenaPointer(12345),
                iteratee: ArenaPointer(67890),
            })
            .as_bytes(),
            [TermTypeDiscriminants::MapIterator as u32, 12345, 67890],
        );
    }
}
