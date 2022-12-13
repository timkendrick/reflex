// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{DependencyList, GraphNode, NilTermType, RefType, SerializeJson, StackOffset};
use serde_json::Value as JsonValue;

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    term_type::TypedTerm,
    ArenaRef,
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct NilTerm;
impl TermSize for NilTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for NilTerm {
    fn hash(&self, hasher: TermHasher, _arena: &impl ArenaAllocator) -> TermHasher {
        hasher
    }
}

impl<'heap, A: ArenaAllocator> NilTermType for ArenaRef<'heap, NilTerm, A> {}

impl<'heap, A: ArenaAllocator> NilTermType for ArenaRef<'heap, TypedTerm<NilTerm>, A> {}

impl<'heap, A: ArenaAllocator> GraphNode for ArenaRef<'heap, NilTerm, A> {
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

impl<'heap, A: ArenaAllocator> SerializeJson for ArenaRef<'heap, NilTerm, A> {
    fn to_json(&self) -> Result<JsonValue, String> {
        Ok(JsonValue::Null)
    }
    fn patch(&self, _target: &Self) -> Result<Option<JsonValue>, String> {
        Ok(None)
    }
}

impl<'heap, A: ArenaAllocator> PartialEq for ArenaRef<'heap, NilTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        true
    }
}
impl<'heap, A: ArenaAllocator> Eq for ArenaRef<'heap, NilTerm, A> {}

impl<'heap, A: ArenaAllocator> std::fmt::Debug for ArenaRef<'heap, NilTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_deref(), f)
    }
}

impl<'heap, A: ArenaAllocator> std::fmt::Display for ArenaRef<'heap, NilTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "null")
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn nil() {
        assert_eq!(
            TermType::Nil(NilTerm).as_bytes(),
            [TermTypeDiscriminants::Nil as u32],
        );
    }
}
