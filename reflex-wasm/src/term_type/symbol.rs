// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{
    DependencyList, GraphNode, SerializeJson, StackOffset, SymbolId, SymbolTermType,
};
use serde_json::Value as JsonValue;

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    term_type::TypedTerm,
    ArenaRef,
};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct SymbolTerm {
    pub id: u32,
}
impl TermSize for SymbolTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for SymbolTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.id, arena)
    }
}

impl<A: ArenaAllocator + Clone> ArenaRef<SymbolTerm, A> {
    pub fn id(&self) -> u32 {
        self.read_value(|term| term.id)
    }
}

impl<A: ArenaAllocator + Clone> SymbolTermType for ArenaRef<SymbolTerm, A> {
    fn id(&self) -> SymbolId {
        self.id() as SymbolId
    }
}

impl<A: ArenaAllocator + Clone> SymbolTermType for ArenaRef<TypedTerm<SymbolTerm>, A> {
    fn id(&self) -> SymbolId {
        <ArenaRef<SymbolTerm, A> as SymbolTermType>::id(&self.as_inner())
    }
}

impl<A: ArenaAllocator + Clone> GraphNode for ArenaRef<SymbolTerm, A> {
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

impl<A: ArenaAllocator + Clone> SerializeJson for ArenaRef<SymbolTerm, A> {
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

impl<A: ArenaAllocator + Clone> PartialEq for ArenaRef<SymbolTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}
impl<A: ArenaAllocator + Clone> Eq for ArenaRef<SymbolTerm, A> {}

impl<A: ArenaAllocator + Clone> std::fmt::Debug for ArenaRef<SymbolTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: ArenaAllocator + Clone> std::fmt::Display for ArenaRef<SymbolTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<symbol:{:#016x}>", self.id())
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn symbol() {
        assert_eq!(
            TermType::Symbol(SymbolTerm { id: 12345 }).as_bytes(),
            [TermTypeDiscriminants::Symbol as u32, 12345],
        );
    }
}
