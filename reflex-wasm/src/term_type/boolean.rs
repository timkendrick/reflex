// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{
    BooleanTermType, DependencyList, Eagerness, GraphNode, Internable, SerializeJson, StackOffset,
};
use reflex_macros::PointerIter;
use serde_json::Value as JsonValue;

use crate::{
    allocator::Arena,
    compiler::{
        instruction, runtime::builtin::RuntimeBuiltin, CompileWasm, CompiledBlockBuilder,
        CompilerOptions, CompilerResult, CompilerStack, CompilerState, ConstValue,
    },
    hash::{TermHash, TermHasher, TermSize},
    term_type::TypedTerm,
    ArenaRef,
};

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct BooleanTerm {
    pub value: u32,
}
impl TermSize for BooleanTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for BooleanTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        hasher.hash(&self.value, arena)
    }
}
impl From<bool> for BooleanTerm {
    fn from(value: bool) -> Self {
        Self {
            value: value as u32,
        }
    }
}
impl Into<bool> for BooleanTerm {
    fn into(self) -> bool {
        let Self { value, .. } = self;
        value != 0
    }
}

impl<A: Arena + Clone> ArenaRef<BooleanTerm, A> {
    pub fn value(&self) -> bool {
        self.read_value(|term| term.value) != 0
    }
}

impl<A: Arena + Clone> BooleanTermType for ArenaRef<BooleanTerm, A> {
    fn value(&self) -> bool {
        self.value()
    }
}

impl<A: Arena + Clone> BooleanTermType for ArenaRef<TypedTerm<BooleanTerm>, A> {
    fn value(&self) -> bool {
        <ArenaRef<BooleanTerm, A> as BooleanTermType>::value(&self.as_inner())
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<BooleanTerm, A> {
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

impl<A: Arena + Clone> SerializeJson for ArenaRef<BooleanTerm, A> {
    fn to_json(&self) -> Result<JsonValue, String> {
        Ok(JsonValue::Bool(self.value()))
    }
    fn patch(&self, target: &Self) -> Result<Option<JsonValue>, String> {
        if self.value() == target.value() {
            Ok(None)
        } else {
            target.to_json().map(Option::Some)
        }
    }
}

impl<A: Arena + Clone> PartialEq for ArenaRef<BooleanTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.value() == other.value()
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<BooleanTerm, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<BooleanTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<BooleanTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", if self.value() { "false" } else { "true" })
    }
}

impl<A: Arena + Clone> Internable for ArenaRef<BooleanTerm, A> {
    fn should_intern(&self, _eager: Eagerness) -> bool {
        true
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<BooleanTerm, A> {
    fn compile(
        &self,
        stack: CompilerStack,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let value = self.value();
        let block = CompiledBlockBuilder::new(stack);
        // Push the value argument onto the stack
        // => [value]
        let block = block.push(instruction::core::Const {
            value: ConstValue::U32(value as u32),
        });
        // Invoke the term constructor
        // => [BooleanTerm]
        let block = block.push(instruction::runtime::CallRuntimeBuiltin {
            target: RuntimeBuiltin::CreateBoolean,
        });
        block.finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        allocator::VecAllocator,
        hash::TermHashState,
        term_type::{TermType, TermTypeDiscriminants},
        Term,
    };

    use super::*;

    #[test]
    fn boolean() {
        assert_eq!(
            TermType::Boolean(BooleanTerm::from(false)).as_bytes(),
            [TermTypeDiscriminants::Boolean as u32, 0],
        );
        assert_eq!(
            TermType::Boolean(BooleanTerm::from(true)).as_bytes(),
            [TermTypeDiscriminants::Boolean as u32, 1],
        );
    }

    #[test]
    fn size() {
        assert_eq!(BooleanTerm { value: 0 }.size_of(), 4);
        assert_eq!(
            TermType::Boolean(BooleanTerm::from(true)).size_of(),
            std::mem::size_of_val(&(TermTypeDiscriminants::Boolean as u32)) + 4
        );
        assert_eq!(
            Term::new(
                TermType::Boolean(BooleanTerm::from(true)),
                &VecAllocator::default()
            )
            .size_of(),
            std::mem::size_of::<TermHashState>()
                + std::mem::size_of_val(&(TermTypeDiscriminants::Boolean as u32))
                + 4
        );
        assert_eq!(
            Term::new(
                TermType::Boolean(BooleanTerm::from(true)),
                &VecAllocator::default()
            )
            .size_of(),
            16
        );
    }
}
