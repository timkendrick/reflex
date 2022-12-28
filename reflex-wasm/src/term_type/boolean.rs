// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{BooleanTermType, DependencyList, GraphNode, SerializeJson, StackOffset};
use serde_json::Value as JsonValue;

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    term_type::TypedTerm,
    ArenaRef,
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct BooleanTerm {
    pub value: u32,
}
impl TermSize for BooleanTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for BooleanTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.value, arena)
    }
}
impl From<bool> for BooleanTerm {
    fn from(value: bool) -> Self {
        Self {
            value: value as u32,
        }
    }
}
impl Into<bool> for BooleanTerm {
    fn into(self) -> bool {
        let Self { value, .. } = self;
        value != 0
    }
}

impl<A: ArenaAllocator + Clone> ArenaRef<BooleanTerm, A> {
    pub fn value(&self) -> bool {
        self.as_value().value != 0
    }
}

impl<A: ArenaAllocator + Clone> BooleanTermType for ArenaRef<BooleanTerm, A> {
    fn value(&self) -> bool {
        self.value()
    }
}

impl<A: ArenaAllocator + Clone> BooleanTermType for ArenaRef<TypedTerm<BooleanTerm>, A> {
    fn value(&self) -> bool {
        <ArenaRef<BooleanTerm, A> as BooleanTermType>::value(&self.as_inner())
    }
}

impl<A: ArenaAllocator + Clone> GraphNode for ArenaRef<BooleanTerm, A> {
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

impl<A: ArenaAllocator + Clone> SerializeJson for ArenaRef<BooleanTerm, A> {
    fn to_json(&self) -> Result<JsonValue, String> {
        Ok(JsonValue::Bool(self.value()))
    }
    fn patch(&self, target: &Self) -> Result<Option<JsonValue>, String> {
        if self.value() == target.value() {
            Ok(None)
        } else {
            target.to_json().map(Option::Some)
        }
    }
}

impl<A: ArenaAllocator + Clone> PartialEq for ArenaRef<BooleanTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
    }
}
impl<A: ArenaAllocator + Clone> Eq for ArenaRef<BooleanTerm, A> {}

impl<A: ArenaAllocator + Clone> std::fmt::Debug for ArenaRef<BooleanTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_value(), f)
    }
}

impl<A: ArenaAllocator + Clone> std::fmt::Display for ArenaRef<BooleanTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", if self.value() { "false" } else { "true" })
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn boolean() {
        assert_eq!(
            TermType::Boolean(BooleanTerm::from(false)).as_bytes(),
            [TermTypeDiscriminants::Boolean as u32, 0],
        );
        assert_eq!(
            TermType::Boolean(BooleanTerm::from(true)).as_bytes(),
            [TermTypeDiscriminants::Boolean as u32, 1],
        );
    }
}
