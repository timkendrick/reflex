// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{DependencyList, GraphNode, IntTermType, IntValue, SerializeJson, StackOffset};
use serde_json::Value as JsonValue;

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    term_type::TypedTerm,
    ArenaRef,
};

use reflex_macros::PointerIter;

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct IntTerm {
    pub value: i32,
}
impl TermSize for IntTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for IntTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.value, arena)
    }
}
impl From<i32> for IntTerm {
    fn from(value: i32) -> Self {
        Self { value }
    }
}
impl Into<i32> for IntTerm {
    fn into(self) -> i32 {
        let Self { value } = self;
        value
    }
}

impl<A: ArenaAllocator + Clone> ArenaRef<IntTerm, A> {
    pub fn value(&self) -> i32 {
        self.read_value(|term| term.value)
    }
}

impl<A: ArenaAllocator + Clone> IntTermType for ArenaRef<IntTerm, A> {
    fn value(&self) -> IntValue {
        self.value()
    }
}

impl<A: ArenaAllocator + Clone> IntTermType for ArenaRef<TypedTerm<IntTerm>, A> {
    fn value(&self) -> IntValue {
        <ArenaRef<IntTerm, A> as IntTermType>::value(&self.as_inner())
    }
}

impl<A: ArenaAllocator + Clone> GraphNode for ArenaRef<IntTerm, A> {
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

impl<A: ArenaAllocator + Clone> SerializeJson for ArenaRef<IntTerm, A> {
    fn to_json(&self) -> Result<JsonValue, String> {
        Ok(JsonValue::Number(self.value().into()))
    }
    fn patch(&self, target: &Self) -> Result<Option<JsonValue>, String> {
        if self.value() == target.value() {
            Ok(None)
        } else {
            target.to_json().map(Some)
        }
    }
}

impl<A: ArenaAllocator + Clone> PartialEq for ArenaRef<IntTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
    }
}
impl<A: ArenaAllocator + Clone> Eq for ArenaRef<IntTerm, A> {}

impl<A: ArenaAllocator + Clone> std::fmt::Debug for ArenaRef<IntTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: ArenaAllocator + Clone> std::fmt::Display for ArenaRef<IntTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn int() {
        assert_eq!(
            TermType::Int(IntTerm::from(12345)).as_bytes(),
            [TermTypeDiscriminants::Int as u32, 12345],
        );
    }
}
