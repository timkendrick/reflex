// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{DependencyList, GraphNode, SerializeJson, StackOffset};
use serde_json::Value as JsonValue;

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    ArenaRef, Term, TermPointer,
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct FilterIteratorTerm {
    pub source: TermPointer,
    pub predicate: TermPointer,
}
impl TermSize for FilterIteratorTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for FilterIteratorTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher
            .hash(&self.source, arena)
            .hash(&self.predicate, arena)
    }
}

impl<A: ArenaAllocator + Clone> ArenaRef<FilterIteratorTerm, A> {
    pub fn source(&self) -> ArenaRef<Term, A> {
        ArenaRef::<Term, _>::new(self.arena.clone(), self.as_value().source)
    }
    pub fn predicate(&self) -> ArenaRef<Term, A> {
        ArenaRef::<Term, _>::new(self.arena.clone(), self.as_value().predicate)
    }
}

impl<A: ArenaAllocator + Clone> SerializeJson for ArenaRef<FilterIteratorTerm, A> {
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

impl<A: ArenaAllocator + Clone> PartialEq for ArenaRef<FilterIteratorTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.source() == other.source() && other.predicate() == other.predicate()
    }
}
impl<A: ArenaAllocator + Clone> Eq for ArenaRef<FilterIteratorTerm, A> {}

impl<A: ArenaAllocator + Clone> std::fmt::Debug for ArenaRef<FilterIteratorTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_value(), f)
    }
}

impl<A: ArenaAllocator + Clone> std::fmt::Display for ArenaRef<FilterIteratorTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FilterIterator")
    }
}

impl<A: ArenaAllocator + Clone> GraphNode for ArenaRef<FilterIteratorTerm, A> {
    fn size(&self) -> usize {
        1 + self.source().size() + self.predicate().size()
    }
    fn capture_depth(&self) -> StackOffset {
        self.source()
            .capture_depth()
            .max(self.predicate().capture_depth())
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
                .chain(self.predicate().dynamic_dependencies(deep))
                .collect()
        } else {
            DependencyList::empty()
        }
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        if deep {
            self.source().has_dynamic_dependencies(deep)
                || self.predicate().has_dynamic_dependencies(deep)
        } else {
            false
        }
    }
    fn is_static(&self) -> bool {
        true
    }
    fn is_atomic(&self) -> bool {
        self.source().is_atomic() && self.predicate().is_atomic()
    }
    fn is_complex(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn filter_iterator() {
        assert_eq!(
            TermType::FilterIterator(FilterIteratorTerm {
                source: TermPointer(12345),
                predicate: TermPointer(67890),
            })
            .as_bytes(),
            [TermTypeDiscriminants::FilterIterator as u32, 12345, 67890],
        );
    }
}
