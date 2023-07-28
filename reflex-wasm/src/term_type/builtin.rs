// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::{
    core::{
        Arity, BuiltinTermType, CompiledFunctionTermType, DependencyList, Expression, GraphNode,
        SerializeJson, StackOffset,
    },
    hash::HashId,
};
use reflex_macros::PointerIter;
use serde_json::Value as JsonValue;

use crate::{
    allocator::Arena,
    compiler::{
        error::CompilerError, instruction, runtime::builtin::RuntimeBuiltin, CompileWasm,
        CompiledBlockBuilder, CompilerOptions, CompilerResult, CompilerStack, CompilerState,
        ConstValue, Eagerness, FunctionPointer, Internable,
    },
    hash::{TermHash, TermHasher, TermSize},
    stdlib::Stdlib,
    term_type::TypedTerm,
    ArenaRef, FunctionIndex, Term, TermType,
};

#[derive(Clone, Copy, Debug, PointerIter)]
#[repr(C)]
pub struct BuiltinTerm {
    pub uid: u32,
}
impl TermSize for BuiltinTerm {
    fn size_of(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
impl TermHash for BuiltinTerm {
    fn hash(&self, hasher: TermHasher, arena: &impl Arena) -> TermHasher {
        hasher.hash(&self.uid, arena)
    }
}
impl From<Stdlib> for BuiltinTerm {
    fn from(value: Stdlib) -> Self {
        Self { uid: value.into() }
    }
}
impl std::fmt::Display for BuiltinTerm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.uid, f)
    }
}

impl<A: Arena + Clone> ArenaRef<BuiltinTerm, A> {
    pub fn target(&self) -> FunctionIndex {
        self.read_value(|term| FunctionIndex::from(term.uid))
    }
    pub fn arity(&self) -> Option<Arity> {
        Stdlib::try_from(u32::from(self.target()))
            .ok()
            .map(|builtin| builtin.arity())
    }
}

impl<T: Expression, A: Arena + Clone> BuiltinTermType<T> for ArenaRef<BuiltinTerm, A>
where
    T::Builtin: From<Stdlib>,
{
    fn target<'a>(&'a self) -> T::Builtin
    where
        T: 'a,
        T::Builtin: 'a,
    {
        T::Builtin::from(self.target().as_stdlib().expect("Invalid function index"))
    }
}

impl<T: Expression, A: Arena + Clone> BuiltinTermType<T> for ArenaRef<TypedTerm<BuiltinTerm>, A>
where
    T::Builtin: From<Stdlib>,
{
    fn target<'a>(&'a self) -> T::Builtin
    where
        T: 'a,
        T::Builtin: 'a,
    {
        <ArenaRef<BuiltinTerm, A> as BuiltinTermType<T>>::target(&self.as_inner())
    }
}

impl<A: Arena + Clone> CompiledFunctionTermType for ArenaRef<BuiltinTerm, A> {
    fn address(&self) -> reflex::core::InstructionPointer {
        self.read_value(|term| reflex::core::InstructionPointer::new(term.uid as usize))
    }
    fn hash(&self) -> HashId {
        self.read_value(|term| {
            let arena = self.arena();
            Term::new(TermType::Builtin(*term), arena).id()
        })
    }
    fn required_args(&self) -> StackOffset {
        match self.arity() {
            Some(arity) => arity.required().len(),
            None => panic!("Unable to retrieve arity of compiled function term"),
        }
    }
    fn optional_args(&self) -> StackOffset {
        match self.arity() {
            Some(arity) => arity.optional().len(),
            None => panic!("Unable to retrieve arity of compiled function term"),
        }
    }
    fn variadic_args(&self) -> bool {
        match self.arity() {
            Some(arity) => arity.variadic().is_some(),
            None => panic!("Unable to retrieve arity of compiled function term"),
        }
    }
}

impl<A: Arena + Clone> CompiledFunctionTermType for ArenaRef<TypedTerm<BuiltinTerm>, A> {
    fn address(&self) -> reflex::core::InstructionPointer {
        <ArenaRef<BuiltinTerm, A> as CompiledFunctionTermType>::address(&self.as_inner())
    }
    fn hash(&self) -> HashId {
        <ArenaRef<BuiltinTerm, A> as CompiledFunctionTermType>::hash(&self.as_inner())
    }
    fn required_args(&self) -> StackOffset {
        <ArenaRef<BuiltinTerm, A> as CompiledFunctionTermType>::required_args(&self.as_inner())
    }
    fn optional_args(&self) -> StackOffset {
        <ArenaRef<BuiltinTerm, A> as CompiledFunctionTermType>::optional_args(&self.as_inner())
    }
    fn variadic_args(&self) -> bool {
        <ArenaRef<BuiltinTerm, A> as CompiledFunctionTermType>::variadic_args(&self.as_inner())
    }
}

impl<A: Arena + Clone> GraphNode for ArenaRef<BuiltinTerm, A> {
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

impl<A: Arena + Clone> SerializeJson for ArenaRef<BuiltinTerm, A> {
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

impl<A: Arena + Clone> PartialEq for ArenaRef<BuiltinTerm, A> {
    fn eq(&self, other: &Self) -> bool {
        self.target() == other.target()
    }
}
impl<A: Arena + Clone> Eq for ArenaRef<BuiltinTerm, A> {}

impl<A: Arena + Clone> std::fmt::Debug for ArenaRef<BuiltinTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.read_value(|term| std::fmt::Debug::fmt(term, f))
    }
}

impl<A: Arena + Clone> std::fmt::Display for ArenaRef<BuiltinTerm, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.target(), f)
    }
}

impl<A: Arena + Clone> Internable for ArenaRef<BuiltinTerm, A> {
    fn should_intern(&self, _eager: Eagerness) -> bool {
        true
    }
}

impl<A: Arena + Clone> CompileWasm<A> for ArenaRef<BuiltinTerm, A> {
    fn compile(
        &self,
        stack: CompilerStack,
        _state: &mut CompilerState,
        _options: &CompilerOptions,
    ) -> CompilerResult<A> {
        let target = self.target();
        let stdlib_target = target
            .as_stdlib()
            .ok_or_else(|| CompilerError::InvalidFunctionTarget(target))?;
        let block = CompiledBlockBuilder::new(stack);
        // Push the function index argument onto the stack
        // => [index]
        let block = block.push(instruction::core::Const {
            value: ConstValue::FunctionPointer(FunctionPointer::Stdlib(stdlib_target)),
        });
        // Invoke the term constructor
        // => [BuiltinTerm]
        let block = block.push(instruction::runtime::CallRuntimeBuiltin {
            target: RuntimeBuiltin::CreateBuiltin,
        });
        block.finish()
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
