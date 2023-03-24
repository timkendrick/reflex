// SPDX-FileCopyrightText: 2023 Marshall Wace <opensource@mwam.com>
// SPDX-License-Identifier: Apache-2.0
// SPDX-FileContributor: Tim Kendrick <t.kendrick@mwam.com> https://github.com/timkendrickmw
// SPDX-FileContributor: Jordan Hall <j.hall@mwam.com> https://github.com/j-hall-mwam
use std::collections::HashSet;

use reflex::core::{
    Arity, BuiltinTermType, DependencyList, Eagerness, Expression, GraphNode, Internable,
    SerializeJson, StackOffset,
};
use reflex_macros::PointerIter;
use serde_json::Value as JsonValue;

use crate::{
    allocator::Arena,
    compiler::{
        builtin::RuntimeBuiltin, CompileWasm, CompiledBlock, CompiledInstruction, CompilerError,
        CompilerOptions, CompilerResult, CompilerStack, CompilerState, CompilerVariableBindings,
    },
    hash::{TermHash, TermHasher, TermSize},
    stdlib::Stdlib,
    term_type::TypedTerm,
    ArenaRef, FunctionIndex,
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
        _state: &mut CompilerState,
        _bindings: &CompilerVariableBindings,
        _options: &CompilerOptions,
        _stack: &CompilerStack,
    ) -> CompilerResult<A> {
        let target = self.target();
        let stdlib = target
            .as_stdlib()
            .ok_or_else(|| CompilerError::InvalidFunctionTarget(target))?;
        let mut instructions = CompiledBlock::default();
        // Push the function index argument onto the stack
        // => [index]
        instructions.push(CompiledInstruction::function_pointer(stdlib));
        // Invoke the term constructor
        // => [BuiltinTerm]
        instructions.push(CompiledInstruction::CallRuntimeBuiltin(
            RuntimeBuiltin::CreateBuiltin,
        ));
        Ok(instructions)
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
