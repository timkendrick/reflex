// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{
    Arity, BuiltinTermType, DependencyList, Expression, GraphNode, RefType, SerializeJson,
    StackOffset,
};
use serde_json::Value as JsonValue;

use crate::{
    allocator::ArenaAllocator,
    hash::{TermHash, TermHasher, TermSize},
    stdlib::Stdlib,
    term_type::TypedTerm,
    ArenaRef,
};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(C)]
pub struct FunctionIndex(u32);
impl TermSize for FunctionIndex {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for FunctionIndex {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        let Self(uid) = self;
        hasher.hash(uid, arena)
    }
}
impl From<FunctionIndex> for u32 {
    fn from(value: FunctionIndex) -> Self {
        let FunctionIndex(value) = value;
        value
    }
}
impl From<Stdlib> for FunctionIndex {
    fn from(value: Stdlib) -> Self {
        Self(u32::from(value))
    }
}
impl std::fmt::Display for FunctionIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match Stdlib::try_from(self.0) {
            Ok(stdlib) => write!(f, "{}", stdlib),
            Err(_) => write!(f, "<unknown:{}>", self.0),
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(C)]
pub struct BuiltinTerm {
    pub uid: FunctionIndex,
}
impl TermSize for BuiltinTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for BuiltinTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl ArenaAllocator) -> TermHasher {
        hasher.hash(&self.uid, arena)
    }
}
impl From<Stdlib> for BuiltinTerm {
    fn from(value: Stdlib) -> Self {
        Self { uid: value.into() }
    }
}

impl<'heap, A: ArenaAllocator> ArenaRef<'heap, BuiltinTerm, A> {
    pub fn target(&self) -> FunctionIndex {
        self.as_deref().uid
    }
    pub fn arity(&self) -> Option<Arity> {
        Stdlib::try_from(u32::from(self.target()))
            .ok()
            .map(|builtin| builtin.arity())
    }
}

impl<'heap, T: Expression, A: ArenaAllocator> BuiltinTermType<T> for ArenaRef<'heap, BuiltinTerm, A>
where
    T::Builtin: From<FunctionIndex>,
{
    fn target<'a>(&'a self) -> T::Builtin
    where
        T: 'a,
        T::Builtin: 'a,
    {
        T::Builtin::from(self.target())
    }
}

impl<'heap, T: Expression, A: ArenaAllocator> BuiltinTermType<T>
    for ArenaRef<'heap, TypedTerm<BuiltinTerm>, A>
where
    T::Builtin: From<FunctionIndex>,
{
    fn target<'a>(&'a self) -> T::Builtin
    where
        T: 'a,
        T::Builtin: 'a,
    {
        <ArenaRef<'heap, BuiltinTerm, A> as BuiltinTermType<T>>::target(&self.as_inner())
    }
}

impl<'heap, A: ArenaAllocator> GraphNode for ArenaRef<'heap, BuiltinTerm, A> {
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

impl<'heap, A: ArenaAllocator> SerializeJson for ArenaRef<'heap, BuiltinTerm, A> {
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

impl<'heap, A: ArenaAllocator> PartialEq for ArenaRef<'heap, BuiltinTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.target() == other.target()
    }
}
impl<'heap, A: ArenaAllocator> Eq for ArenaRef<'heap, BuiltinTerm, A> {}

impl<'heap, A: ArenaAllocator> std::fmt::Debug for ArenaRef<'heap, BuiltinTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_deref(), f)
    }
}

impl<'heap, A: ArenaAllocator> std::fmt::Display for ArenaRef<'heap, BuiltinTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<stdlib:{:?}>", self.target())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        stdlib::{Add, StdlibDiscriminants},
        term_type::{TermType, TermTypeDiscriminants},
    };

    use super::*;

    #[test]
    fn builtin() {
        assert_eq!(
            TermType::Builtin(BuiltinTerm::from(Stdlib::from(Add))).as_bytes(),
            [
                TermTypeDiscriminants::Builtin as u32,
                StdlibDiscriminants::Add as u32
            ],
        );
    }
}
