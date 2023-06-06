// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{
    DependencyList, Eagerness, EffectTermType, Expression, GraphNode, Internable, SerializeJson,
    StackOffset,
};
use serde_json::Value as JsonValue;

use crate::{
    allocator::Arena,
    hash::{TermHash, TermHasher, TermSize},
    term_type::TypedTerm,
    ArenaPointer, ArenaRef,
};
use reflex_macros::PointerIter;

use super::{ConditionTerm, WasmExpression};

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct EffectTerm {
    pub condition: ArenaPointer,
}
impl TermSize for EffectTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for EffectTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        hasher.hash(&self.condition, arena)
    }
}

impl<A: Arena + Clone> ArenaRef<EffectTerm, A> {
    pub fn condition(&self) -> ArenaRef<TypedTerm<ConditionTerm>, A> {
        ArenaRef::<TypedTerm<ConditionTerm>, _>::new(
            self.arena.clone(),
            self.read_value(|term| term.condition),
        )
    }
}

impl<A: Arena + Clone> EffectTermType<WasmExpression<A>> for ArenaRef<EffectTerm, A> {
    fn condition<'a>(&'a self) -> <WasmExpression<A> as Expression>::SignalRef<'a>
    where
        <WasmExpression<A> as Expression>::Signal: 'a,
        WasmExpression<A>: 'a,
    {
        self.condition().into()
    }
}

impl<A: Arena + Clone> EffectTermType<WasmExpression<A>> for ArenaRef<TypedTerm<EffectTerm>, A> {
    fn condition<'a>(&'a self) -> <WasmExpression<A> as Expression>::SignalRef<'a>
    where
        <WasmExpression<A> as Expression>::Signal: 'a,
        WasmExpression<A>: 'a,
    {
        <ArenaRef<EffectTerm, A> as EffectTermType<WasmExpression<A>>>::condition(&self.as_inner())
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<EffectTerm, A> {
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
        DependencyList::of(self.condition().read_value(|condition| condition.id()))
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

impl<A: Arena + Clone> SerializeJson for ArenaRef<EffectTerm, A> {
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

impl<A: Arena + Clone> PartialEq for ArenaRef<EffectTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.condition() == other.condition()
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<EffectTerm, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<EffectTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<EffectTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<effect:{}>", self.condition())
    }
}

impl<A: Arena + Clone> Internable for ArenaRef<EffectTerm, A> {
    fn should_intern(&self, eager: Eagerness) -> bool {
        eager == Eagerness::Lazy && self.capture_depth() == 0
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
                condition: ArenaPointer(0x54321)
            })
            .as_bytes(),
            [TermTypeDiscriminants::Effect as u32, 0x54321],
        );
    }
}
