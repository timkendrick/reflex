// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::{collections::HashSet, iter::once};

use reflex::core::{DependencyList, GraphNode, SerializeJson, StackOffset, VariableTermType};
use serde_json::Value as JsonValue;

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    term_type::TypedTerm,
    ArenaRef,
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct VariableTerm {
    pub stack_offset: u32,
}
impl TermSize for VariableTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for VariableTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.stack_offset, arena)
    }
}

impl<A: ArenaAllocator + Clone> ArenaRef<VariableTerm, A> {
    pub fn stack_offset(&self) -> StackOffset {
        self.read_value(|term| term.stack_offset as StackOffset)
    }
}

impl<A: ArenaAllocator + Clone> VariableTermType for ArenaRef<VariableTerm, A> {
    fn offset(&self) -> StackOffset {
        self.stack_offset()
    }
}

impl<A: ArenaAllocator + Clone> VariableTermType for ArenaRef<TypedTerm<VariableTerm>, A> {
    fn offset(&self) -> StackOffset {
        <ArenaRef<VariableTerm, A> as VariableTermType>::offset(&self.as_inner())
    }
}

impl<A: ArenaAllocator + Clone> GraphNode for ArenaRef<VariableTerm, A> {
    fn size(&self) -> usize {
        1
    }
    fn capture_depth(&self) -> StackOffset {
        (self.stack_offset()) + 1
    }
    fn free_variables(&self) -> HashSet<StackOffset> {
        HashSet::from_iter(once(self.stack_offset()))
    }
    fn count_variable_usages(&self, offset: StackOffset) -> usize {
        if offset == (self.stack_offset()) {
            1
        } else {
            0
        }
    }
    fn dynamic_dependencies(&self, _deep: bool) -> DependencyList {
        DependencyList::empty()
    }
    fn has_dynamic_dependencies(&self, _deep: bool) -> bool {
        false
    }
    fn is_static(&self) -> bool {
        false
    }
    fn is_atomic(&self) -> bool {
        false
    }
    fn is_complex(&self) -> bool {
        false
    }
}

impl<A: ArenaAllocator + Clone> SerializeJson for ArenaRef<VariableTerm, A> {
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

impl<A: ArenaAllocator + Clone> PartialEq for ArenaRef<VariableTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.stack_offset() == other.stack_offset()
    }
}
impl<A: ArenaAllocator + Clone> Eq for ArenaRef<VariableTerm, A> {}

impl<A: ArenaAllocator + Clone> std::fmt::Debug for ArenaRef<VariableTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: ArenaAllocator + Clone> std::fmt::Display for ArenaRef<VariableTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<variable:{}>", self.stack_offset())
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn variable() {
        assert_eq!(
            TermType::Variable(VariableTerm {
                stack_offset: 12345,
            })
            .as_bytes(),
            [TermTypeDiscriminants::Variable as u32, 12345],
        );
    }
}
