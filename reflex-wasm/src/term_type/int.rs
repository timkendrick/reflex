// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{
    DependencyList, Eagerness, GraphNode, IntTermType, IntValue, Internable, SerializeJson,
    StackOffset,
};
use serde_json::Value as JsonValue;

use crate::{
    allocator::Arena,
    hash::{TermHash, TermHasher, TermSize},
    term_type::TypedTerm,
    utils::{chunks_to_i64, i64_to_chunks},
    ArenaRef,
};

use reflex_macros::PointerIter;

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct IntTerm {
    pub value: [u32; 2],
}
impl TermSize for IntTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for IntTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        hasher.hash(&self.value, arena)
    }
}
impl From<i64> for IntTerm {
    fn from(value: i64) -> Self {
        Self {
            value: i64_to_chunks(value),
        }
    }
}
impl Into<i64> for IntTerm {
    fn into(self) -> i64 {
        let Self { value } = self;
        chunks_to_i64(value)
    }
}

impl<A: Arena + Clone> ArenaRef<IntTerm, A> {
    pub fn value(&self) -> i64 {
        self.read_value(|term| chunks_to_i64(term.value))
    }
}

impl<A: Arena + Clone> IntTermType for ArenaRef<IntTerm, A> {
    fn value(&self) -> IntValue {
        self.value()
    }
}

impl<A: Arena + Clone> IntTermType for ArenaRef<TypedTerm<IntTerm>, A> {
    fn value(&self) -> IntValue {
        <ArenaRef<IntTerm, A> as IntTermType>::value(&self.as_inner())
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<IntTerm, A> {
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

impl<A: Arena + Clone> SerializeJson for ArenaRef<IntTerm, A> {
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

impl<A: Arena + Clone> PartialEq for ArenaRef<IntTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<IntTerm, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<IntTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<IntTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value())
    }
}

impl<A: Arena + Clone> Internable for ArenaRef<IntTerm, A> {
    fn should_intern(&self, _eager: Eagerness) -> bool {
        self.capture_depth() == 0
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn int() {
        assert_eq!(
            TermType::Int(IntTerm::from(0x987654321)).as_bytes(),
            [TermTypeDiscriminants::Int as u32, 0x87654321, 0x00000009],
        );
        assert_eq!(TermType::Int(IntTerm::from(-0x987654321)).as_bytes(), {
            let [low, high] = i64_to_chunks(-0x987654321);
            [TermTypeDiscriminants::Int as u32, low, high]
        },);
    }
}
