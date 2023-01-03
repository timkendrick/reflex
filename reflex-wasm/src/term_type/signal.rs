// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{
    DependencyList, Expression, GraphNode, SerializeJson, SignalTermType, StackOffset,
};
use serde_json::Value as JsonValue;

use super::{TreeTerm, WasmExpression};
use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    term_type::TypedTerm,
    ArenaRef, TermPointer,
};
use reflex_macros::PointerIter;

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct SignalTerm {
    pub conditions: TermPointer,
}
impl TermSize for SignalTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for SignalTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.conditions, arena)
    }
}

impl<A: ArenaAllocator + Clone> ArenaRef<SignalTerm, A> {
    pub fn conditions(&self) -> ArenaRef<TypedTerm<TreeTerm>, A> {
        ArenaRef::<TypedTerm<TreeTerm>, _>::new(
            self.arena.clone(),
            self.read_value(|term| term.conditions),
        )
    }
}

impl<A: ArenaAllocator + Clone> SignalTermType<WasmExpression<A>> for ArenaRef<SignalTerm, A> {
    fn signals<'a>(&'a self) -> <WasmExpression<A> as Expression>::SignalListRef<'a>
    where
        <WasmExpression<A> as Expression>::SignalList: 'a,
        WasmExpression<A>: 'a,
    {
        self.conditions().into()
    }
}

impl<A: ArenaAllocator + Clone> SignalTermType<WasmExpression<A>>
    for ArenaRef<TypedTerm<SignalTerm>, A>
{
    fn signals<'a>(&'a self) -> <WasmExpression<A> as Expression>::SignalListRef<'a>
    where
        <WasmExpression<A> as Expression>::SignalList: 'a,
        WasmExpression<A>: 'a,
    {
        <ArenaRef<SignalTerm, A> as SignalTermType<WasmExpression<A>>>::signals(&self.as_inner())
    }
}

impl<A: ArenaAllocator + Clone> GraphNode for ArenaRef<SignalTerm, A> {
    fn size(&self) -> usize {
        1 + (self.conditions().as_inner().len() as usize)
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
        false
    }
    fn is_complex(&self) -> bool {
        false
    }
}

impl<A: ArenaAllocator + Clone> SerializeJson for ArenaRef<SignalTerm, A> {
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

impl<A: ArenaAllocator + Clone> PartialEq for ArenaRef<SignalTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.conditions() == other.conditions()
    }
}
impl<A: ArenaAllocator + Clone> Eq for ArenaRef<SignalTerm, A> {}

impl<A: ArenaAllocator + Clone> std::fmt::Debug for ArenaRef<SignalTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: ArenaAllocator + Clone> std::fmt::Display for ArenaRef<SignalTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}]",
            self.conditions()
                .as_inner()
                .nodes()
                .map(|effect| format!("{}", effect))
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn signal() {
        assert_eq!(
            TermType::Signal(SignalTerm {
                conditions: TermPointer(12345),
            })
            .as_bytes(),
            [TermTypeDiscriminants::Signal as u32, 12345],
        );
    }
}
