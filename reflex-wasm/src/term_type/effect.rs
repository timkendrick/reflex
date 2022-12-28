// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{
    DependencyList, EffectTermType, Expression, GraphNode, SerializeJson, StackOffset,
};
use serde_json::Value as JsonValue;

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    term_type::TypedTerm,
    ArenaRef, TermPointer,
};

use super::{ConditionTerm, WasmExpression};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct EffectTerm {
    pub condition: TermPointer,
}
impl TermSize for EffectTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for EffectTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.condition, arena)
    }
}

impl<A: ArenaAllocator + Clone> ArenaRef<EffectTerm, A> {
    pub fn condition(&self) -> ArenaRef<TypedTerm<ConditionTerm>, A> {
        ArenaRef::<TypedTerm<ConditionTerm>, _>::new(self.arena.clone(), self.as_value().condition)
    }
}

impl<A: ArenaAllocator + Clone> EffectTermType<WasmExpression<A>> for ArenaRef<EffectTerm, A> {
    fn condition<'a>(&'a self) -> <WasmExpression<A> as Expression>::SignalRef<'a>
    where
        <WasmExpression<A> as Expression>::Signal: 'a,
        WasmExpression<A>: 'a,
    {
        self.condition().into()
    }
}

impl<A: ArenaAllocator + Clone> EffectTermType<WasmExpression<A>>
    for ArenaRef<TypedTerm<EffectTerm>, A>
{
    fn condition<'a>(&'a self) -> <WasmExpression<A> as Expression>::SignalRef<'a>
    where
        <WasmExpression<A> as Expression>::Signal: 'a,
        WasmExpression<A>: 'a,
    {
        <ArenaRef<EffectTerm, A> as EffectTermType<WasmExpression<A>>>::condition(&self.as_inner())
    }
}

impl<A: ArenaAllocator + Clone> GraphNode for ArenaRef<EffectTerm, A> {
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
        DependencyList::of(self.condition().as_value().id())
    }
    fn has_dynamic_dependencies(&self, _deep: bool) -> bool {
        true
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

impl<A: ArenaAllocator + Clone> SerializeJson for ArenaRef<EffectTerm, A> {
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

impl<A: ArenaAllocator + Clone> PartialEq for ArenaRef<EffectTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.condition() == other.condition()
    }
}
impl<A: ArenaAllocator + Clone> Eq for ArenaRef<EffectTerm, A> {}

impl<A: ArenaAllocator + Clone> std::fmt::Debug for ArenaRef<EffectTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_value(), f)
    }
}

impl<A: ArenaAllocator + Clone> std::fmt::Display for ArenaRef<EffectTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<effect:{}>", self.condition())
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn effect() {
        assert_eq!(
            TermType::Effect(EffectTerm {
                condition: TermPointer(12345)
            })
            .as_bytes(),
            [TermTypeDiscriminants::Effect as u32, 12345],
        );
    }
}
