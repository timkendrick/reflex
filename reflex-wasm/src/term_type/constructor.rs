// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{
    Arity, ConstructorTermType, DependencyList, Expression, GraphNode, SerializeJson, StackOffset,
};
use serde_json::Value as JsonValue;

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    term_type::TypedTerm,
    ArenaRef, TermPointer,
};
use reflex_macros::PointerIter;

use super::{ListTerm, WasmExpression};

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct ConstructorTerm {
    pub keys: TermPointer,
}
impl TermSize for ConstructorTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for ConstructorTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.keys, arena)
    }
}

impl<A: ArenaAllocator + Clone> ArenaRef<ConstructorTerm, A> {
    pub fn keys(&self) -> ArenaRef<TypedTerm<ListTerm>, A> {
        ArenaRef::<TypedTerm<ListTerm>, _>::new(
            self.arena.clone(),
            self.read_value(|term| term.keys),
        )
    }
    pub fn arity(&self) -> Arity {
        Arity::lazy(self.keys().as_inner().len(), 0, false)
    }
}

impl<A: ArenaAllocator + Clone> ConstructorTermType<WasmExpression<A>>
    for ArenaRef<ConstructorTerm, A>
{
    fn prototype<'a>(&'a self) -> <WasmExpression<A> as Expression>::StructPrototypeRef<'a>
    where
        <WasmExpression<A> as Expression>::StructPrototype: 'a,
        WasmExpression<A>: 'a,
    {
        self.keys().into()
    }
}

impl<A: ArenaAllocator + Clone> ConstructorTermType<WasmExpression<A>>
    for ArenaRef<TypedTerm<ConstructorTerm>, A>
{
    fn prototype<'a>(&'a self) -> <WasmExpression<A> as Expression>::StructPrototypeRef<'a>
    where
        <WasmExpression<A> as Expression>::StructPrototype: 'a,
        WasmExpression<A>: 'a,
    {
        <ArenaRef<ConstructorTerm, A> as ConstructorTermType<WasmExpression<A>>>::prototype(
            &self.as_inner(),
        )
    }
}

impl<A: ArenaAllocator + Clone> GraphNode for ArenaRef<ConstructorTerm, A> {
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

impl<A: ArenaAllocator + Clone> SerializeJson for ArenaRef<ConstructorTerm, A> {
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

impl<A: ArenaAllocator + Clone> PartialEq for ArenaRef<ConstructorTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.keys() == other.keys()
    }
}
impl<A: ArenaAllocator + Clone> Eq for ArenaRef<ConstructorTerm, A> {}

impl<A: ArenaAllocator + Clone> std::fmt::Debug for ArenaRef<ConstructorTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: ArenaAllocator + Clone> std::fmt::Display for ArenaRef<ConstructorTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<constructor:{{{}}}>", self.keys())
    }
}

#[cfg(test)]
mod tests {
    use crate::term_type::{TermType, TermTypeDiscriminants};

    use super::*;

    #[test]
    fn constructor() {
        assert_eq!(
            TermType::Constructor(ConstructorTerm {
                keys: TermPointer(12345)
            })
            .as_bytes(),
            [TermTypeDiscriminants::Constructor as u32, 12345],
        );
    }
}
