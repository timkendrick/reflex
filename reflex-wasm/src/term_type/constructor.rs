// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{
    Arity, ConstructorTermType, DependencyList, Expression, GraphNode, RefType, SerializeJson,
    StackOffset,
};
use serde_json::Value as JsonValue;

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    term_type::TypedTerm,
    ArenaRef, TermPointer,
};

use super::ListTerm;

#[derive(Clone, Copy, Debug)]
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

impl<'heap, A: ArenaAllocator> ArenaRef<'heap, ConstructorTerm, A> {
    pub fn keys(&self) -> ArenaRef<'heap, TypedTerm<ListTerm>, A> {
        ArenaRef::new(self.arena, self.arena.get(self.as_deref().keys))
    }
    pub fn arity(&self) -> Arity {
        Arity::lazy(self.keys().as_inner().len(), 0, false)
    }
}

impl<'heap, T: Expression, A: ArenaAllocator> ConstructorTermType<T>
    for ArenaRef<'heap, ConstructorTerm, A>
where
    for<'a> T::StructPrototypeRef<'a, T>: From<ArenaRef<'a, TypedTerm<ListTerm>, A>>,
{
    fn prototype<'a>(&'a self) -> T::StructPrototypeRef<'a, T>
    where
        T::StructPrototype<T>: 'a,
        T: 'a,
    {
        self.keys().into()
    }
}

impl<'heap, T: Expression, A: ArenaAllocator> ConstructorTermType<T>
    for ArenaRef<'heap, TypedTerm<ConstructorTerm>, A>
where
    for<'a> T::StructPrototypeRef<'a, T>: From<ArenaRef<'a, TypedTerm<ListTerm>, A>>,
{
    fn prototype<'a>(&'a self) -> T::StructPrototypeRef<'a, T>
    where
        T::StructPrototype<T>: 'a,
        T: 'a,
    {
        <ArenaRef<'heap, ConstructorTerm, A> as ConstructorTermType<T>>::prototype(&self.as_inner())
    }
}

impl<'heap, A: ArenaAllocator> GraphNode for ArenaRef<'heap, ConstructorTerm, A> {
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

impl<'heap, A: ArenaAllocator> SerializeJson for ArenaRef<'heap, ConstructorTerm, A> {
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

impl<'heap, A: ArenaAllocator> PartialEq for ArenaRef<'heap, ConstructorTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.keys() == other.keys()
    }
}
impl<'heap, A: ArenaAllocator> Eq for ArenaRef<'heap, ConstructorTerm, A> {}

impl<'heap, A: ArenaAllocator> std::fmt::Debug for ArenaRef<'heap, ConstructorTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_deref(), f)
    }
}

impl<'heap, A: ArenaAllocator> std::fmt::Display for ArenaRef<'heap, ConstructorTerm, A> {
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
