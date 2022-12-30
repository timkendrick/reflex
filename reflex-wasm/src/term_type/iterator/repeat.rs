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
pub struct RepeatIteratorTerm {
    pub value: TermPointer,
}
impl TermSize for RepeatIteratorTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for RepeatIteratorTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.value, arena)
    }
}

impl<A: ArenaAllocator + Clone> ArenaRef<RepeatIteratorTerm, A> {
    pub fn value(&self) -> ArenaRef<Term, A> {
        ArenaRef::<Term, _>::new(self.arena.clone(), self.read_value(|term| term.value))
    }
}

impl<A: ArenaAllocator + Clone> SerializeJson for ArenaRef<RepeatIteratorTerm, A> {
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

impl<A: ArenaAllocator + Clone> PartialEq for ArenaRef<RepeatIteratorTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
    }
}
impl<A: ArenaAllocator + Clone> Eq for ArenaRef<RepeatIteratorTerm, A> {}

impl<A: ArenaAllocator + Clone> std::fmt::Debug for ArenaRef<RepeatIteratorTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: ArenaAllocator + Clone> std::fmt::Display for ArenaRef<RepeatIteratorTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RepeatIterator")
    }
}

impl<A: ArenaAllocator + Clone> GraphNode for ArenaRef<RepeatIteratorTerm, A> {
    fn size(&self) -> usize {
        1 + self.value().size()
    }
    fn capture_depth(&self) -> StackOffset {
        self.value().capture_depth()
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        self.value().free_variables()
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        self.value().count_variable_usages(offset)
    }
    fn dynamic_dependencies(&self, deep: bool) -> DependencyList {
        if deep {
            self.value().dynamic_dependencies(deep)
        } else {
            DependencyList::empty()
        }
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        if deep {
            self.value().has_dynamic_dependencies(deep)
        } else {
            false
        }
    }
    fn is_static(&self) -> bool {
        true
    }
    fn is_atomic(&self) -> bool {
        self.value().is_atomic()
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
    fn repeat_iterator() {
        assert_eq!(
            TermType::RepeatIterator(RepeatIteratorTerm {
                value: TermPointer(12345),
            })
            .as_bytes(),
            [TermTypeDiscriminants::RepeatIterator as u32, 12345],
        );
    }
}
