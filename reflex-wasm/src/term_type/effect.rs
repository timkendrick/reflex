// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{
    DependencyList, EffectTermType, Expression, GraphNode, RefType, SerializeJson, StackOffset,
};
use serde_json::Value as JsonValue;

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    term_type::TypedTerm,
    ArenaRef, TermPointer,
};

use super::ConditionTerm;

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

impl<'heap, A: ArenaAllocator> ArenaRef<'heap, EffectTerm, A> {
    pub fn condition(&self) -> ArenaRef<'heap, TypedTerm<ConditionTerm>, A> {
        ArenaRef::new(self.arena, self.arena.get(self.as_deref().condition))
    }
}

impl<'heap, T: Expression, A: ArenaAllocator> EffectTermType<T> for ArenaRef<'heap, EffectTerm, A>
where
    for<'a> T::SignalRef<'a, T>: From<ArenaRef<'a, TypedTerm<ConditionTerm>, A>>,
{
    fn condition<'a>(&'a self) -> T::SignalRef<'a, T>
    where
        T::Signal<T>: 'a,
        T: 'a,
    {
        self.condition().into()
    }
}

impl<'heap, T: Expression, A: ArenaAllocator> EffectTermType<T>
    for ArenaRef<'heap, TypedTerm<EffectTerm>, A>
where
    for<'a> T::SignalRef<'a, T>: From<ArenaRef<'a, TypedTerm<ConditionTerm>, A>>,
{
    fn condition<'a>(&'a self) -> T::SignalRef<'a, T>
    where
        T::Signal<T>: 'a,
        T: 'a,
    {
        <ArenaRef<'heap, EffectTerm, A> as EffectTermType<T>>::condition(&self.as_inner())
    }
}

impl<'heap, A: ArenaAllocator> GraphNode for ArenaRef<'heap, EffectTerm, A> {
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
        DependencyList::of(self.condition().as_deref().id())
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

impl<'heap, A: ArenaAllocator> SerializeJson for ArenaRef<'heap, EffectTerm, A> {
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

impl<'heap, A: ArenaAllocator> PartialEq for ArenaRef<'heap, EffectTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.condition() == other.condition()
    }
}
impl<'heap, A: ArenaAllocator> Eq for ArenaRef<'heap, EffectTerm, A> {}

impl<'heap, A: ArenaAllocator> std::fmt::Debug for ArenaRef<'heap, EffectTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_deref(), f)
    }
}

impl<'heap, A: ArenaAllocator> std::fmt::Display for ArenaRef<'heap, EffectTerm, A> {
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
