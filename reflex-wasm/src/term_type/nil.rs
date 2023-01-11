// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{
    CompiledFunctionTermType, DependencyList, Eagerness, Expression, GraphNode, Internable,
    NilTermType, RecursiveTermType, SerializeJson, StackOffset,
};
use serde_json::Value as JsonValue;

use crate::{
    allocator::Arena,
    hash::{TermHash, TermHasher, TermSize},
    term_type::TypedTerm,
    ArenaRef,
};
use reflex_macros::PointerIter;

use super::WasmExpression;

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct NilTerm;
impl TermSize for NilTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for NilTerm {
    fn hash(&self, hasher: TermHasher, _arena: &impl Arena) -> TermHasher {
        hasher
    }
}

impl<A: Arena + Clone> NilTermType for ArenaRef<NilTerm, A> {}

impl<A: Arena + Clone> NilTermType for ArenaRef<TypedTerm<NilTerm>, A> {}

// FIXME: implement RecursiveTerm
impl<A: Arena + Clone> RecursiveTermType<WasmExpression<A>> for ArenaRef<TypedTerm<NilTerm>, A> {
    fn factory<'a>(&'a self) -> <WasmExpression<A> as Expression>::ExpressionRef<'a>
    where
        WasmExpression<A>: 'a,
    {
        panic!("Recursive terms not currently supported")
    }
}

// FIXME: remove CompiledFunctionTerm
impl<A: Arena + Clone> CompiledFunctionTermType for ArenaRef<TypedTerm<NilTerm>, A> {
    fn address(&self) -> reflex::core::InstructionPointer {
        panic!("Compiled function terms not supported")
    }
    fn hash(&self) -> reflex::hash::HashId {
        panic!("Compiled function terms not supported")
    }
    fn required_args(&self) -> StackOffset {
        panic!("Compiled function terms not supported")
    }
    fn optional_args(&self) -> StackOffset {
        panic!("Compiled function terms not supported")
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<NilTerm, A> {
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

impl<A: Arena + Clone> SerializeJson for ArenaRef<NilTerm, A> {
    fn to_json(&self) -> Result<JsonValue, String> {
        Ok(JsonValue::Null)
    }
    fn patch(&self, _target: &Self) -> Result<Option<JsonValue>, String> {
        Ok(None)
    }
}

impl<A: Arena + Clone> PartialEq for ArenaRef<NilTerm, A> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<NilTerm, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<NilTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<NilTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "null")
    }
}

impl<A: Arena + Clone> Internable for ArenaRef<NilTerm, A> {
    fn should_intern(&self, _eager: Eagerness) -> bool {
        self.capture_depth() == 0
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
