// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{DependencyList, Eagerness, GraphNode, Internable, SerializeJson, StackOffset};
use serde_json::Value as JsonValue;

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    ArenaRef,
};
use reflex_macros::PointerIter;

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct RangeIteratorTerm {
    pub offset: i32,
    pub length: u32,
}
impl TermSize for RangeIteratorTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for RangeIteratorTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.offset, arena).write_u32(self.length)
    }
}

impl<A: ArenaAllocator + Clone> ArenaRef<RangeIteratorTerm, A> {
    pub fn offset(&self) -> i32 {
        self.read_value(|term| term.offset)
    }
    pub fn length(&self) -> u32 {
        self.read_value(|term| term.length)
    }
}

impl<A: ArenaAllocator + Clone> SerializeJson for ArenaRef<RangeIteratorTerm, A> {
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

impl<A: ArenaAllocator + Clone> PartialEq for ArenaRef<RangeIteratorTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.offset() == other.offset() && self.length() == other.length()
    }
}
impl<A: ArenaAllocator + Clone> Eq for ArenaRef<RangeIteratorTerm, A> {}

impl<A: ArenaAllocator + Clone> std::fmt::Debug for ArenaRef<RangeIteratorTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: ArenaAllocator + Clone> std::fmt::Display for ArenaRef<RangeIteratorTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RangeIterator")
    }
}

impl<A: ArenaAllocator + Clone> GraphNode for ArenaRef<RangeIteratorTerm, A> {
    fn size(&self) -> usize {
        1
    }
    fn capture_depth(&self) -> StackOffset {
        0
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        HashSet::new()
    }
    fn count_variable_usages(&self, _offset: StackOffset) -> usize {
        0
    }
    fn dynamic_dependencies(&self, _deep: bool) -> DependencyList {
        DependencyList::empty()
    }
    fn has_dynamic_dependencies(&self, _deep: bool) -> bool {
        false
    }
    fn is_static(&self) -> bool {
        true
    }
    fn is_atomic(&self) -> bool {
        true
    }
    fn is_complex(&self) -> bool {
        false
    }
}

impl<A: ArenaAllocator + Clone> Internable for ArenaRef<RangeIteratorTerm, A> {
    fn should_intern(&self, _eager: Eagerness) -> bool {
        self.capture_depth() == 0
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    fn twos_complement(value: i32) -> u32 {
        if value >= 0 {
            value as u32
        } else {
            0xFFFFFFFF - ((value.abs() - 1) as u32)
        }
    }

    #[test]
    fn range_iterator() {
        assert_eq!(
            TermType::RangeIterator(RangeIteratorTerm {
                offset: 12345,
                length: 67890,
            })
            .as_bytes(),
            [TermTypeDiscriminants::RangeIterator as u32, 12345, 67890],
        );
        assert_eq!(
            TermType::RangeIterator(RangeIteratorTerm {
                offset: -12345,
                length: 67890,
            })
            .as_bytes(),
            [
                TermTypeDiscriminants::RangeIterator as u32,
                twos_complement(-12345),
                67890
            ],
        );
    }
}
