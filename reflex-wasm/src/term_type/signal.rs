// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{
    DependencyList, Expression, GraphNode, SerializeJson, SignalTermType, StackOffset,
};
use serde_json::Value as JsonValue;

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    term_type::TypedTerm,
    ArenaRef, TermPointer,
};

use super::{TreeTerm, WasmExpression};

#[derive(Clone, Copy, Debug)]
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

impl<'heap, A: ArenaAllocator> ArenaRef<'heap, SignalTerm, A> {
    fn conditions(&self) -> ArenaRef<'heap, TypedTerm<TreeTerm>, A> {
        ArenaRef::<TypedTerm<TreeTerm>, _>::new(self.arena, self.as_value().conditions)
    }
}

impl<'heap, A: ArenaAllocator> SignalTermType<WasmExpression<'heap, A>>
    for ArenaRef<'heap, SignalTerm, A>
{
    fn signals<'a>(&'a self) -> <WasmExpression<'heap, A> as Expression>::SignalListRef<'a>
    where
        <WasmExpression<'heap, A> as Expression>::SignalList: 'a,
        WasmExpression<'heap, A>: 'a,
    {
        self.conditions().into()
    }
}

impl<'heap, A: ArenaAllocator> SignalTermType<WasmExpression<'heap, A>>
    for ArenaRef<'heap, TypedTerm<SignalTerm>, A>
{
    fn signals<'a>(&'a self) -> <WasmExpression<'heap, A> as Expression>::SignalListRef<'a>
    where
        <WasmExpression<'heap, A> as Expression>::SignalList: 'a,
        WasmExpression<'heap, A>: 'a,
    {
        <ArenaRef<'heap, SignalTerm, A> as SignalTermType<WasmExpression<'heap, A>>>::signals(
            &self.as_inner(),
        )
    }
}

impl<'heap, A: ArenaAllocator> GraphNode for ArenaRef<'heap, SignalTerm, A> {
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

impl<'heap, A: ArenaAllocator> SerializeJson for ArenaRef<'heap, SignalTerm, A> {
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

impl<'heap, A: ArenaAllocator> PartialEq for ArenaRef<'heap, SignalTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.conditions() == other.conditions()
    }
}
impl<'heap, A: ArenaAllocator> Eq for ArenaRef<'heap, SignalTerm, A> {}

impl<'heap, A: ArenaAllocator> std::fmt::Debug for ArenaRef<'heap, SignalTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_value(), f)
    }
}

impl<'heap, A: ArenaAllocator> std::fmt::Display for ArenaRef<'heap, SignalTerm, A> {
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
