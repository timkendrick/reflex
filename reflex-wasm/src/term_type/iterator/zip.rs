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
pub struct ZipIteratorTerm {
    pub left: TermPointer,
    pub right: TermPointer,
}
impl TermSize for ZipIteratorTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for ZipIteratorTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.left, arena).hash(&self.right, arena)
    }
}

impl<A: ArenaAllocator + Clone> ArenaRef<ZipIteratorTerm, A> {
    pub fn left(&self) -> ArenaRef<Term, A> {
        ArenaRef::<Term, _>::new(self.arena.clone(), self.read_value(|term| term.left))
    }
    pub fn right(&self) -> ArenaRef<Term, A> {
        ArenaRef::<Term, _>::new(self.arena.clone(), self.read_value(|term| term.right))
    }
}

impl<A: ArenaAllocator + Clone> SerializeJson for ArenaRef<ZipIteratorTerm, A> {
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

impl<A: ArenaAllocator + Clone> PartialEq for ArenaRef<ZipIteratorTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.left() == other.left() && self.right() == other.right()
    }
}
impl<A: ArenaAllocator + Clone> Eq for ArenaRef<ZipIteratorTerm, A> {}

impl<A: ArenaAllocator + Clone> std::fmt::Debug for ArenaRef<ZipIteratorTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: ArenaAllocator + Clone> std::fmt::Display for ArenaRef<ZipIteratorTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ZipIterator")
    }
}

impl<A: ArenaAllocator + Clone> GraphNode for ArenaRef<ZipIteratorTerm, A> {
    fn size(&self) -> usize {
        1 + self.left().size() + self.right().size()
    }
    fn capture_depth(&self) -> StackOffset {
        self.left()
            .capture_depth()
            .max(self.right().capture_depth())
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        self.left().free_variables()
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        self.left().count_variable_usages(offset)
    }
    fn dynamic_dependencies(&self, deep: bool) -> DependencyList {
        if deep {
            self.left()
                .dynamic_dependencies(deep)
                .into_iter()
                .chain(self.right().dynamic_dependencies(deep))
                .collect()
        } else {
            DependencyList::empty()
        }
    }
    fn has_dynamic_dependencies(&self, deep: bool) -> bool {
        if deep {
            self.left().has_dynamic_dependencies(deep)
                || self.right().has_dynamic_dependencies(deep)
        } else {
            false
        }
    }
    fn is_static(&self) -> bool {
        true
    }
    fn is_atomic(&self) -> bool {
        self.left().is_atomic() && self.right().is_atomic()
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
    fn zip_iterator() {
        assert_eq!(
            TermType::ZipIterator(ZipIteratorTerm {
                left: TermPointer(12345),
                right: TermPointer(67890),
            })
            .as_bytes(),
            [TermTypeDiscriminants::ZipIterator as u32, 12345, 67890],
        );
    }
}
