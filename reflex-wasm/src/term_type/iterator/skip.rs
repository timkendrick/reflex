// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{DependencyList, GraphNode, RefType, SerializeJson, StackOffset};
use serde_json::Value as JsonValue;

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    ArenaRef, Term, TermPointer,
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SkipIteratorTerm {
    pub source: TermPointer,
    pub count: u32,
}
impl TermSize for SkipIteratorTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for SkipIteratorTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.source, arena).write_u32(self.count)
    }
}

impl<'heap, A: ArenaAllocator> ArenaRef<'heap, SkipIteratorTerm, A> {
    fn source(&self) -> ArenaRef<'heap, Term, A> {
        ArenaRef::new(self.arena, self.arena.get(self.as_deref().source))
    }
    fn count(&self) -> u32 {
        self.as_deref().count
    }
}

impl<'heap, A: ArenaAllocator> SerializeJson for ArenaRef<'heap, SkipIteratorTerm, A> {
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

impl<'heap, A: ArenaAllocator> PartialEq for ArenaRef<'heap, SkipIteratorTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.source() == other.source() && self.count() == other.count()
    }
}
impl<'heap, A: ArenaAllocator> Eq for ArenaRef<'heap, SkipIteratorTerm, A> {}

impl<'heap, A: ArenaAllocator> std::fmt::Debug for ArenaRef<'heap, SkipIteratorTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_deref(), f)
    }
}

impl<'heap, A: ArenaAllocator> std::fmt::Display for ArenaRef<'heap, SkipIteratorTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SkipIterator")
    }
}

impl<'heap, A: ArenaAllocator> GraphNode for ArenaRef<'heap, SkipIteratorTerm, A> {
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
    fn skip_iterator() {
        assert_eq!(
            TermType::SkipIterator(SkipIteratorTerm {
                source: TermPointer(12345),
                count: 67890,
            })
            .as_bytes(),
            [TermTypeDiscriminants::SkipIterator as u32, 12345, 67890],
        );
    }
}
