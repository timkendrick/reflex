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
use reflex_macros::PointerIter;

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct TakeIteratorTerm {
    pub source: TermPointer,
    pub count: u32,
}
impl TermSize for TakeIteratorTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for TakeIteratorTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.source, arena).write_u32(self.count)
    }
}

impl<A: ArenaAllocator + Clone> ArenaRef<TakeIteratorTerm, A> {
    pub fn source(&self) -> ArenaRef<Term, A> {
        ArenaRef::<Term, _>::new(self.arena.clone(), self.read_value(|term| term.source))
    }
    pub fn count(&self) -> u32 {
        self.read_value(|term| term.count)
    }
}

impl<A: ArenaAllocator + Clone> SerializeJson for ArenaRef<TakeIteratorTerm, A> {
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

impl<A: ArenaAllocator + Clone> PartialEq for ArenaRef<TakeIteratorTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.source() == other.source() && self.count() == other.count()
    }
}
impl<A: ArenaAllocator + Clone> Eq for ArenaRef<TakeIteratorTerm, A> {}

impl<A: ArenaAllocator + Clone> std::fmt::Debug for ArenaRef<TakeIteratorTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: ArenaAllocator + Clone> std::fmt::Display for ArenaRef<TakeIteratorTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TakeIterator")
    }
}

impl<A: ArenaAllocator + Clone> GraphNode for ArenaRef<TakeIteratorTerm, A> {
    fn size(&self) -> usize {
        1 + self.source().size()
    }
    fn capture_depth(&self) -> StackOffset {
        self.source().capture_depth()
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        self.source().free_variables()
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        self.source().count_variable_usages(offset)
    }
    fn dynamic_dependencies(&self, deep: bool) -> DependencyList {
        if deep {
            self.source().dynamic_dependencies(deep)
        } else {
            DependencyList::empty()
        }
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        if deep {
            self.source().has_dynamic_dependencies(deep)
        } else {
            false
        }
    }
    fn is_static(&self) -> bool {
        true
    }
    fn is_atomic(&self) -> bool {
        self.source().is_atomic()
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
    fn take_iterator() {
        assert_eq!(
            TermType::TakeIterator(TakeIteratorTerm {
                source: TermPointer(12345),
                count: 67890,
            })
            .as_bytes(),
            [TermTypeDiscriminants::TakeIterator as u32, 12345, 67890],
        );
    }
}
